use std::collections::{BTreeMap,VecDeque};

use super::futures::*;
use super::futures::stream::Stream;

use super::hyper;
use hyper::header::{Raw};
use hyper::{Request,Method};

use connector::navigator::navigation_rules::NavigationRules;
use connector::navigator::Navigator;
use connector::{Connector,HeaderContentType};
use configure::*;
use result::*;

pub mod extraction_rules;
pub mod specific_procedure;
use self::extraction_rules::{ExtractionRules,UnitExtractionRule,ProcedureRule,ProcedureName};
use self::specific_procedure::{SpecificProcedure,DefaultProcedure,NaverWebtoonProcedure};

pub struct Extractor<'a, 'b> {
    rules: &'a ExtractionRules,
    config: &'b ExtractorConfigure,
}

impl<'a, 'b> Extractor<'a, 'b> {
    pub fn new(rule_config: &'a ExtractionRules,
               system_config: &'b ExtractorConfigure) -> Extractor<'a, 'b> {
        Extractor {
            rules: rule_config,
            config: system_config,
        }
    }

    pub fn extract_all(&self) {
        let extraction_rules = self.rules.extraction();
        for rule in extraction_rules {
            self.extract(rule);
        }
    }

    fn extract(&self, rule: &UnitExtractionRule) {
        let mut conn = Connector::new();
        Extractor::set_entry_links(&mut conn, rule);

        let request_urls = conn.request_urls();
        let request_urls = &request_urls;
        let extract_contents_result =
            conn.run_request_all(|index, response| {
                let source_url = &request_urls[index];
                let unit_response_handler =
                    UnitExtractionRuleResponseHandler::new(index, source_url.clone(), rule);
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

                    debug!("Extractor::extract - body_content : {}", (&body_content).as_str());

                    let part_contents = unit_response_handler.part_from_content(body_content);
                    let extract_contents: Vec<ResultHandler> =
                        unit_response_handler.extract_from_parts(part_contents, raw_cookies);

                    future::ok(extract_contents)
                })
            });

        match extract_contents_result {
            Ok(extract_contents) => {
                Extractor::handle_results(extract_contents, rule);
            },
            Err(err) => {
                debug!("Extractor::extract - error occurred : {}", err);
            },
        }
    }

    fn handle_results(results: Vec<Vec<ResultHandler>>, rule: &UnitExtractionRule) {
        use std::fs;
        use std::io::Write;
        use std::path::{PathBuf};

        let mut conn = Connector::new();

        let index_path_map =
            Extractor::ready_for_request(&mut conn, &results, rule);
        let extraction_results =
            conn.run_request_all(|index, response| {
            let path = index_path_map.get(&index).unwrap();

            let extension = match Connector::get_content_type(&response) {
                HeaderContentType::Image(ref ext) => ext.clone(),
                HeaderContentType::Text(ref ext) => ext.clone(),
                HeaderContentType::Others(ref _ext) => "unknown".to_owned(),
            };

            let mut file_name = String::new();
            file_name.push_str(path.as_str());
            file_name.push_str(".");
            file_name.push_str(extension.as_str());

            //FIXME: save file
            let path = PathBuf::from(file_name.clone());
            let parent_path = path.parent().unwrap();
            if !parent_path.exists() {
                match fs::create_dir_all(parent_path) {
                    Err(err) => {
                        debug!("save_file - error occurred : {}", err);
                    },
                    Ok(_) => {},
                }
            }

            let mut file = fs::File::create(file_name).expect("file error");
            response.body().for_each(move |chunk| {
                file.write_all(&chunk).map_err(From::from)
            })
        });
    }

    fn ready_for_request(conn: &mut Connector,
                         extract_targets: &Vec<Vec<ResultHandler>>,
                         rule: &UnitExtractionRule) -> BTreeMap<usize, String> {
        let mut index_path_map = BTreeMap::<usize, String>::new();

        let post_procedure_name = match rule.procedure() {
            Some(procedure) => {
                let post_procedure = match procedure.post_procedure() {
                    Some(post_procedure) => {
                        ProcedureRule::procedure_name(post_procedure.as_str())
                    },
                    None => ProcedureName::None,
                };
                post_procedure
            },
            None => ProcedureName::None,
        };

        let mut key_index: usize = 0;
        for extract_target in extract_targets {
            for result_handler in extract_target {
                let procedure =
                    Extractor::get_procedure(result_handler, &post_procedure_name);
                let mut order_index: usize = 0;

                let store_label = ReservedLabel::Store;
                let stores =
                    result_handler.get_result(store_label.to_string().as_str());

                match stores {
                    Some(stores) => {
                        for link in stores {
                            let mut path = String::new();
                            match result_handler.get_root_path() {
                                Some(root_path) => {
                                    path.push_str(root_path.as_str());
                                },
                                None => { },
                            }
                            path.push_str(order_index.to_string().as_str());
                            index_path_map.insert(key_index, path.clone());

                            order_index += 1;
                            key_index += 1;

                            let request = procedure.get_request(link);
                            conn.add_request(request);
                        }
                    },
                    None => continue,
                }
            }
        }
        index_path_map
    }

    fn get_procedure<'lrh, 'lpn>(result_handler: &'lrh ResultHandler,
                                 procedure_name: &'lpn ProcedureName) -> Box<SpecificProcedure + 'lrh> {
        match procedure_name {
            &ProcedureName::NaverWebtoon => Box::new(NaverWebtoonProcedure::new(result_handler)),
            &ProcedureName::None => Box::new(DefaultProcedure::new(result_handler)),
        }
    }

    fn get_req_links(name: &String) -> VecDeque<String> {
        let nav_rules = NavigationRules::default();
        let navigator = Navigator::new(&nav_rules);
        let nav_name_index_map = navigator.name_index_map();

        let index = match nav_name_index_map.get(name) {
            Option::Some(index) => index.clone(),
            Option::None => {
                return VecDeque::new();
            }
        };

        let target_navigation = match nav_rules.navigation().get(index) {
            Option::Some(navigation) => navigation,
            Option::None => {
                return VecDeque::new();
            }
        };
        navigator.navigate(target_navigation)
    }

    fn set_entry_links<'c, 'r>(conn: &'c mut Connector, rule: &'r UnitExtractionRule) {
        match rule.links() {
            Option::Some(links) => {
                for link in links {
                    conn.add_request(Request::new(Method::Get, link.parse().unwrap()));
                }
            },
            Option::None => {
                let links: VecDeque<String> = Extractor::get_req_links(rule.name());
                for link in links {
                    conn.add_request(Request::new(Method::Get, link.parse().unwrap()));
                }
            }
        };
    }
}

struct UnitExtractionRuleResponseHandler<'a> {
    queue_index: usize,
    source_url: String,
    rule: &'a UnitExtractionRule,
}

impl <'a> UnitExtractionRuleResponseHandler<'a> {
    pub fn new(index: usize, source_url: String,
               rule: &'a UnitExtractionRule) -> UnitExtractionRuleResponseHandler {
        UnitExtractionRuleResponseHandler {
            queue_index: index,
            source_url: source_url,
            rule: rule,
        }
    }

    fn part_from_content(&self, content: String) -> Vec<String> {
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

    fn extract_from_parts(&self,
                          part_contents: Vec<String>,
                          raw_cookies: Option<Raw>) -> Vec<ResultHandler> {
        let rule = self.rule;
        let index = self.queue_index;

        let extract_rule = rule.extract();
        let extract_contents: Vec<ResultHandler> = match extract_rule {
            Option::Some(extract_exp) => {
                let root_path =
                    ResultHandler::get_abs_root_path(rule.local_path(),
                                                     rule.name().as_str(),
                                                     index);
                RuleUtils::make_result_handlers(part_contents, extract_exp,
                                                Option::Some(root_path),
                                                self.source_url.clone(), raw_cookies)
            },
            Option::None => {
                let empty_vec: Vec<ResultHandler> = Vec::new();
                empty_vec
            }
        };
        extract_contents
    }
}
