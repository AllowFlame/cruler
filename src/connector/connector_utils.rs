use std::collections::HashMap;

use hyper::{Uri};

pub struct ConnectorUtils { }

impl ConnectorUtils {
    pub fn get_query_value<'a, 'b>(uri: &'a Uri, key: &'b str) -> Option<String> {
        let query = uri.query();
        let query = match query {
            Some(query) => query,
            None => return Option::None,
        };
        let query_map = ConnectorUtils::make_query_map(query);
        match query_map.get(key) {
            Option::Some(value) => Option::Some(value.to_owned()),
            Option::None => Option::None,
        }
    }

    pub fn make_query_map(raw_query: &str) -> HashMap<String, String> {
        let mut query_map = HashMap::new();
        let parsed_query: Vec<&str> = raw_query.split("&").collect();
        for single_query in parsed_query {
            let pair: Vec<&str> = single_query.split("=").collect();
            query_map.insert(pair[0].to_owned(), pair[1].to_owned());
        }
        query_map
    }

    pub fn rebuild_uri_for_adding_query(org_uri: &Uri, add_query: HashMap<String, String>) -> Uri {
        let scheme = org_uri.scheme();
        let authority = org_uri.authority();
        let path = org_uri.path();
        let org_query = org_uri.query();
        //TODO: fragment is not yet supported
        let fragment = Option::None;

        let query_map = match org_query {
            Some(org_query) => {
                let mut query_map = ConnectorUtils::make_query_map(org_query);
                for (key, value) in add_query {
                    query_map.insert(key, value);
                }
                query_map
            },
            None => {
                add_query
            },
        };
        let rebuilt_raw_query = ConnectorUtils::make_raw_query_from_map(query_map);

        ConnectorUtils::build_uri(scheme, authority, path,
                                  Option::Some(rebuilt_raw_query.as_str()), fragment)
    }

    pub fn make_raw_query_from_map(query_map: HashMap<String, String>) -> String {
        let mut raw_query = String::new();
        for (key, value) in query_map {
            raw_query.push_str(key.as_str());
            raw_query.push('=');
            raw_query.push_str(value.as_str());
            raw_query.push('&');
        }
        raw_query.pop();
        raw_query
    }

    pub fn build_uri(scheme: Option<&str>, authority: Option<&str>,
                     path: &str, query: Option<&str>, fragment: Option<&str>) -> Uri {
        let mut built_uri = String::new();
        match scheme {
            Option::Some(scheme) => {
                built_uri.push_str(scheme);
                built_uri.push_str("://");
            },
            Option::None => { },
        }
        match authority {
            Option::Some(authority) => {
                built_uri.push_str(authority);
            },
            Option::None => { },
        }
        built_uri.push_str(path);
        match query {
            Option::Some(query) => {
                built_uri.push('?');
                built_uri.push_str(query);
            },
            Option::None => { },
        }
        match fragment {
            Option::Some(fragment) => {
                built_uri.push('#');
                built_uri.push_str(fragment);
            },
            Option::None => { },
        }

        built_uri.as_str().parse().unwrap()
    }
}