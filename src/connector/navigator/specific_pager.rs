use std::cell::{RefCell};
use std::collections::{HashSet,HashMap,VecDeque};

use hyper::{Uri,Request,Method};
use connector::{Connector};
use connector::core::str::FromStr;
use connector::connector_utils::ConnectorUtils;

use result::*;

pub enum Ordering {
    Ascending,
    Descending,
}

pub trait SpecificPager {
    fn get_entry_uri(&self, entry_url: &str) -> Uri;
    fn is_requested(&self, link: &str) -> bool;
    fn set_as_requested(&self, link: &str);
    fn has_next_request(&self) -> bool;
    fn make_next_requests(&mut self, pager_results: &Vec<ResultHandler>) -> VecDeque<Request>;
    fn collect_ordered_result(&self, label: ReservedLabel,
                              result_handlers: &Vec<ResultHandler>) -> VecDeque<String>;
    fn ordering(&self) -> Ordering;
}

pub struct NaverWebtoonPager {
    request_history: RefCell<HashSet<String>>,
    next_request_available: bool,
}

impl SpecificPager for NaverWebtoonPager {
    fn get_entry_uri(&self, entry_url: &str) -> Uri {
        let input_uri = Uri::from_str(entry_url).unwrap();
        let page_value = ConnectorUtils::get_query_value(&input_uri, "page");
        match page_value {
            Option::Some(_page_num) => {
                input_uri
            },
            Option::None => {
                let mut add_query: HashMap<String, String> = HashMap::new();
                add_query.insert("page".to_owned(), "1".to_owned());
                ConnectorUtils::rebuild_uri_for_adding_query(&input_uri, add_query)
            },
        }
    }

    fn is_requested(&self, link: &str) -> bool {
        let history = self.request_history.borrow();
        history.contains(link)
    }

    fn set_as_requested(&self, link: &str) {
        let relative_uri = NaverWebtoonPager::relative_uri_string(link);
        let encoded_uri = NaverWebtoonPager::encode_escape_char(relative_uri);
        self.insert_history(encoded_uri.as_str());
    }

    fn has_next_request(&self) -> bool {
        self.next_request_available
    }

    fn make_next_requests(&mut self, pager_results: &Vec<ResultHandler>) -> VecDeque<Request> {
        let mut requests = VecDeque::new();

        for result_handler in pager_results {
            let link_label = ReservedLabel::Link.to_string();
            let link_result =
                result_handler.get_result(link_label.as_str());
            match link_result {
                Option::Some(links) => {
                    for link in links {
                        if !self.is_requested(link.as_str()) {
                            let link_string = NaverWebtoonPager::decode_escape_char(link.to_owned());
                            let requestable_uri =
                                result_handler.make_requestable_uri(link_string.as_str());
                            requests.push_back(Request::new(Method::Get, requestable_uri));

                            self.insert_history(link.as_str());

                            debug!("NaverWebtoonPager::make_next_requests - ! requestable_uri : {}", link.as_str());
                        }
                        else {
                            debug!("NaverWebtoonPager::make_next_requests - O requestable_uri : {}", link.as_str());
                        }
                    }
                },
                Option::None => { },
            }

            let type_label = ReservedLabel::Type.to_string();
            let type_result =
                result_handler.get_result(type_label.as_str());
            match type_result {
                Option::Some(types) => {
                    let mut is_next_shown = false;
                    for single_type in types {
                        if single_type.as_str() == "next" {
                            is_next_shown = true;
                        }
                    }
                    //NOTE: if next type is not shown once, it will be the last joined request and must stop to prevent unlimited looping
                    self.next_request_available &= is_next_shown;
                },
                Option::None => { },
            }
        }
        requests
    }

    fn collect_ordered_result(&self, label: ReservedLabel,
                              result_handlers: &Vec<ResultHandler>) -> VecDeque<String> {
        let mut ordered_results = VecDeque::new();

        let label_name = label.to_string();
        let label_name = label_name.as_str();
        for result_handler in result_handlers {
            let source_url = result_handler.get_extra_inform(ExtraInformKey::SourceUrl).unwrap();
            match result_handler.get_result(label_name) {
                Option::Some(result) => {
                    let mut link_result = VecDeque::new();
                    for link in result {
                        let link_uri = result_handler.make_requestable_uri(link.as_str());
                        debug!("NaverWebtoonPager::collect_ordered_result - source_url : {}, link : {}", source_url, link.as_str());
                        link_result.push_back(link_uri.as_ref().to_owned());
                    }
                    ordered_results.append(&mut link_result);
                },
                Option::None => { },
            }
        }

        ordered_results
    }

    fn ordering(&self) -> Ordering {
        Ordering::Descending
    }
}

impl NaverWebtoonPager {
    pub fn new() -> NaverWebtoonPager {
        let history = HashSet::new();

        NaverWebtoonPager {
            request_history: RefCell::new(history),
            next_request_available: true,
        }
    }

    fn insert_history(&self, link: &str) {
        let mut history = self.request_history.borrow_mut();
        history.insert(link.to_owned());
        debug!("NaverWebtoonPager::insert_history - link : {}", link);
    }

    fn relative_uri_string(uri_str: &str) -> String {
        let uri = Uri::from_str(uri_str).unwrap();
        if uri.is_absolute() {
            let path = uri.path();
            let query = uri.query();
            let rebuilt_uri = ConnectorUtils::build_uri(Option::None,
                                                      Option::None,
                                                      path, query, Option::None);
            return rebuilt_uri.as_ref().to_owned();
        }
        uri.as_ref().to_owned()
    }

    fn encode_escape_char(target_string: String) -> String {
        let amp = "&amp;";
        let lt = "&lt;";
        let gt = "&gt;";
        let quot = "&quot;";
        let apos = "&apos;";

        let mut encode_string = target_string.replace("&", amp);
        encode_string = encode_string.replace("<", lt);
        encode_string = encode_string.replace(">", gt);
        encode_string = encode_string.replace("\"", quot);
        encode_string = encode_string.replace("'", apos);

        encode_string
    }

    fn decode_escape_char(target_string: String) -> String {
        let amp = "&";
        let lt = "<";
        let gt = ">";
        let quot = "\"";
        let apos = "'";

        let mut encode_string = target_string.replace("&amp;", amp);
        encode_string = encode_string.replace("&lt;", lt);
        encode_string = encode_string.replace("&gt;", gt);
        encode_string = encode_string.replace("&quot;", quot);
        encode_string = encode_string.replace("&apos;", apos);

        encode_string
    }
}








//TODO: Default Pager is not yet written properly.
pub struct DefaultPager {
    conn: RefCell<Connector>,
}

impl SpecificPager for DefaultPager {
    fn get_entry_uri(&self, entry_url: &str) -> Uri {
        Uri::from_str(entry_url).unwrap()
    }

    fn is_requested(&self, link: &str) -> bool {
        let requests_count = self.conn.borrow().get_requests_count();
        if requests_count == 0 {
            false
        }
        else {
            true
        }
    }

    fn set_as_requested(&self, link: &str) {

    }

    fn has_next_request(&self) -> bool {
        true
    }

    fn make_next_requests(&mut self, pager_results: &Vec<ResultHandler>) -> VecDeque<Request> {
        let requests = VecDeque::new();

        requests
    }

    fn collect_ordered_result(&self, label: ReservedLabel,
                              result_handlers: &Vec<ResultHandler>) -> VecDeque<String> {
        let ordered_results = VecDeque::new();

        let label_name = label.to_string();
        let label_name = label_name.as_str();
        for result_handler in result_handlers {
            let source_url = result_handler.get_extra_inform(ExtraInformKey::SourceUrl).unwrap();
            match result_handler.get_result(label_name) {
                Option::Some(result) => {
                    for link in result {
                        debug!("DefaultPager::source_url : {}, link : {}", source_url, link.as_str());
                    }
                },
                Option::None => { },
            }
        }

        ordered_results
    }

    fn ordering(&self) -> Ordering {
        Ordering::Ascending
    }
}

impl DefaultPager {
    pub fn new() -> DefaultPager {
        let mut conn = Connector::new();

        DefaultPager {
            conn: RefCell::new(conn),
        }
    }

}