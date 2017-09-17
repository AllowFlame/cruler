use std::collections::{HashMap,VecDeque};

use super::hyper;
use super::futures::*;

use hyper::header::{Raw};
use hyper::{Request,Method};

use connector::{Connector,HeaderContentType};
use configure::*;
use result::*;

pub mod navigation_rules;
pub mod specific_pager;
use self::navigation_rules::{NavigationRules,UnitNavigationRule,PagerRule};
use self::specific_pager::{Ordering,SpecificPager,DefaultPager,NaverWebtoonPager};

pub struct Navigator<'a> {
    rules: &'a NavigationRules,
}

impl<'a> Navigator<'a> {
    pub fn new(rule_config: &'a NavigationRules) -> Navigator<'a> {
        Navigator {
            rules: rule_config,
        }
    }

    //FIXME: navigate_all for testing
    pub fn navigate_all(&self) {
        let navigation_rules = self.rules.navigation();
        for rule in navigation_rules {
            self.navigate(rule);
        }
    }

    pub fn name_index_map(&self) -> HashMap<&String, usize> {
        let mut map = HashMap::new();
        let navigation_rules = self.rules.navigation();

        let mut index: usize = 0;
        for rule in navigation_rules {
            let name = rule.name();
            map.insert(name, index);
            index += 1;
        }

        map
    }

    #[inline(always)]
    fn merge_vec(source: &mut VecDeque<String>, target: &mut VecDeque<String>, ordering: Ordering) {
        match ordering {
            Ordering::Ascending => {
                loop {
                    let pop = target.pop_front();
                    match pop {
                        Option::Some(value) => {
                            source.push_back(value);
                        },
                        Option::None => break,
                    }
                }
            },
            Ordering::Descending => {
                loop {
                    let pop = target.pop_front();
                    match pop {
                        Option::Some(value) => {
                            source.push_front(value);
                        },
                        Option::None => break,
                    }
                }
            },
        }
    }

    pub fn navigate(&self, rule: &UnitNavigationRule) -> VecDeque<String> {
        let mut conn = Connector::new();
        let mut pager = Navigator::get_pager(rule);
        let entry_uri = pager.get_entry_uri(rule.entry().as_str());
        pager.set_as_requested(entry_uri.as_ref());
        conn.add_request(Request::new(Method::Get, entry_uri));

        let mut extracted_nav_links = VecDeque::new();
        while pager.has_next_request() {
            let response_result =
                Navigator::run_request(&mut conn, rule);
            conn.clear_requests();

            match response_result {
                Result::Ok(navigation_result_handlers) => {
                    let mut in_page_links =
                        self.get_navigation_links_in_page(&mut conn, &mut pager,
                                                          &navigation_result_handlers);
                    Navigator::merge_vec(&mut extracted_nav_links,
                                         &mut in_page_links, pager.ordering());
                },
                Result::Err(err) => {
                    info!("Navigator::navigate - err : {}", err);
                    continue;
                },
            }
        }

        extracted_nav_links
    }

    fn get_navigation_links_in_page(&self, conn: &mut Connector, pager: &mut Box<SpecificPager>,
                                    handlers: &Vec<NavigationResultHandler>) -> VecDeque<String> {
        let mut links = VecDeque::new();
        for navigation_result_handler in handlers {
            let pager_result =
                navigation_result_handler.pager_result();
            match pager_result {
                Option::Some(pager_result_handlers) => {
                    let mut requests =
                        pager.make_next_requests(pager_result_handlers);
                    conn.add_requests(&mut requests);
                },
                Option::None => { },
            }

            let extracted_results = navigation_result_handler.extracted_results();
            let mut nav_results =
                pager.collect_ordered_result(ReservedLabel::Collect,
                                             extracted_results);

            links.append(&mut nav_results);
        }
        links
    }

    fn run_request(conn: &mut Connector, rule: &UnitNavigationRule)
        -> Result<Vec<NavigationResultHandler>, hyper::Error> {
        let request_urls = conn.request_urls();
        let request_urls = &request_urls;
        conn.run_request_all(|index, response| {
            let source_url = &request_urls[index];
            let unit_response_handler =
                UnitNavigationRuleResponseHandler::new(source_url.clone(), rule);
            let header_type = Connector::get_content_type(&response);
            let will_be_okay = match header_type {
                HeaderContentType::Text(_header) => {
                    true
                },
                _ => {
                    false
                },
            };

            let raw_cookies: Option<Raw> = Connector::get_raw_cookies(&response);

            response.body().fold(Vec::new(), move |mut v, chunk| {
                if !will_be_okay {
                    return future::failed(hyper::Error::Header);
                }
                v.extend(&chunk[..]);
                future::ok::<_, hyper::Error>(v)
            }).and_then(move |chunks| {
                let body_content = match String::from_utf8(chunks) {
                    Ok(body) => body,
                    Err(err) => {
                        return future::failed(hyper::Error::Utf8(err.utf8_error()));
                    }
                };

                let part_contents =
                    unit_response_handler.part_from_content((&body_content).as_str());
                let extract_contents: Vec<ResultHandler> =
                    unit_response_handler.extract_from_content_part(part_contents,
                                                                    raw_cookies.clone());

                let pager_part_contents =
                    unit_response_handler.part_with_pager((&body_content).as_str());
                let pager_results: Vec<ResultHandler> =
                    unit_response_handler.extract_from_pager_part(pager_part_contents,
                                                                  raw_cookies.clone());

                let navigation_result_handler =
                    NavigationResultHandler::new(extract_contents,
                                                 Option::Some(pager_results));

                future::ok(navigation_result_handler)
            })
        })
    }

    fn get_pager(rule: &UnitNavigationRule) -> Box<SpecificPager> {
        match rule.pager() {
            Some(pager) => {
                let pager_name = pager.pager().as_str();
                match pager_name {
                    "naver-webtoon" => Box::new(NaverWebtoonPager::new()),
                    _ => Box::new(DefaultPager::new()),
                }
            },
            None => Box::new(DefaultPager::new()),
        }
    }
}

struct UnitNavigationRuleResponseHandler<'a> {
    source_url: String,
    rule: &'a UnitNavigationRule,
}

impl<'a> UnitNavigationRuleResponseHandler<'a> {
    pub fn new(source_url: String, rule: &'a UnitNavigationRule) -> UnitNavigationRuleResponseHandler {
        UnitNavigationRuleResponseHandler {
            source_url: source_url,
            rule: rule,
        }
    }

    fn part_from_content(&self, cnt: &str) -> Vec<String> {
        let content = cnt.to_owned();
        let part_rules: Option<&Vec<String>> = self.rule.parts();
        let part_contents: Vec<String> = match part_rules {
            Option::Some(part_exps) => {
                RuleUtils::make_part_contents(content, part_exps)
            },
            Option::None => {
                let mut part_content: Vec<String> = Vec::new();
                part_content.push(content);
                part_content
            }
        };
        part_contents
    }

    fn extract_from_content_part(&self,
                                 part_contents: Vec<String>,
                                 raw_cookies: Option<Raw>) -> Vec<ResultHandler> {
        let rule = self.rule;
        let extract_rule = rule.extract();

        self.extract_from_parts(extract_rule, part_contents, raw_cookies)
    }

    fn extract_from_parts(&self, extract_rule: Option<&String>,
                          part_contents: Vec<String>,
                          raw_cookies: Option<Raw>) -> Vec<ResultHandler> {
        let extract_contents: Vec<ResultHandler> = match extract_rule {
            Option::Some(extract_exp) => {
                RuleUtils::make_result_handlers(part_contents, extract_exp,
                                                Option::None,
                                                self.source_url.clone(), raw_cookies)
            },
            Option::None => {
                let empty_vec = Vec::new();
                empty_vec
            }
        };
        extract_contents
    }

    fn part_with_pager(&self, cnt: &str) -> Vec<String> {
        let content = cnt.to_owned();
        let pager_rule: &PagerRule = match self.rule.pager() {
            Option::Some(part_rule) => part_rule,
            Option::None => {
                let part_content = Vec::new();
                return part_content;
            }
        };
        let part_content = match pager_rule.parts() {
            Option::Some(part_exps) => {
                RuleUtils::make_part_contents(content, part_exps)
            },
            Option::None => {
                let mut part_content: Vec<String> = Vec::new();
                part_content.push(content);
                part_content
            }
        };
        part_content
    }

    fn extract_from_pager_part(&self,
                               part_contents: Vec<String>,
                               raw_cookies: Option<Raw>) -> Vec<ResultHandler> {
        let pager_rule: &PagerRule = match self.rule.pager() {
            Option::Some(part_rule) => part_rule,
            Option::None => {
                let extract_content = Vec::new();
                return extract_content;
            }
        };
        let extract_rule = pager_rule.extract();
        self.extract_from_parts(extract_rule, part_contents, raw_cookies)
    }

}

struct NavigationResultHandler {
    extracted_results: Vec<ResultHandler>,
    pager_result: Option<Vec<ResultHandler>>,
}

impl NavigationResultHandler {
    pub fn new(extracted_results: Vec<ResultHandler>,
               pager_result: Option<Vec<ResultHandler>>) -> NavigationResultHandler {
        NavigationResultHandler {
            extracted_results: extracted_results,
            pager_result: pager_result,
        }
    }

    pub fn extracted_results(&self) -> &Vec<ResultHandler> {
        &self.extracted_results
    }

    pub fn pager_result(&self) -> Option<&Vec<ResultHandler>> {
        self.pager_result.as_ref()
    }
}