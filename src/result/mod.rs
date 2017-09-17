#[cfg(test)]
mod result_test;

use std::str::FromStr;
use std::string::{ToString};
use std::collections::HashMap;

use hyper::{Uri};
use hyper::header::Raw;

pub enum ExtraInformKey {
    SourceUrl,
    Reserved(String),
}

impl ExtraInformKey {
    pub fn new(key: &str) -> ExtraInformKey {
        ExtraInformKey::from_str(key).unwrap()
    }
}

impl FromStr for ExtraInformKey {
    type Err = String;
    fn from_str(key: &str) -> Result<Self, Self::Err> {
        if key.to_lowercase() == "SourceUrl".to_lowercase() {
            Result::Ok(ExtraInformKey::SourceUrl)
        }
        else {
            Result::Ok(ExtraInformKey::Reserved(key.to_owned()))
        }
    }
}

impl ToString for ExtraInformKey {
    fn to_string(&self) -> String {
        match self {
            &ExtraInformKey::SourceUrl => {
                "SourceUrl".to_owned()
            },
            &ExtraInformKey::Reserved(ref name) => {
                name.clone()
            },
        }
    }
}

pub struct ResultHandler {
    root_path: Option<String>,
    raw_cookies: Option<Raw>,
    result_map: HashMap<String, Vec<String>>,
    extra_informs: HashMap<String, String>,
}

impl ResultHandler {
    pub fn new(root_path: Option<String>, raw_cookies: Option<Raw>) -> ResultHandler {
        ResultHandler {
            root_path: root_path,
            raw_cookies: raw_cookies,
            result_map: HashMap::new(),
            extra_informs: HashMap::new(),
        }
    }

    pub fn get_root_path(&self) -> Option<&String> {
        self.root_path.as_ref()
    }

    pub fn insert_extra_inform(&mut self, key: ExtraInformKey, value: String) {
        self.extra_informs.insert(key.to_string(), value);
    }

    pub fn get_extra_inform(&self, key: ExtraInformKey) -> Option<&String> {
        self.extra_informs.get(key.to_string().as_str())
    }

    pub fn get_raw_cookies(&self) -> Option<Raw> {
        return self.raw_cookies.clone()
    }

    pub fn get_label_names(&self) -> Vec<String> {
        let result_map = &self.result_map;
        let mut label_names = Vec::new();
        for (label_name, _value) in result_map {
            label_names.push(label_name.clone());
        }
        label_names
    }

    pub fn get_result(&self, label_name: &str) -> Option<&Vec<String>> {
        self.result_map.get(label_name)
    }

    pub fn insert_result(&mut self, key: &str, value: Vec<String>) {
        self.result_map.insert(key.to_owned(), value);
    }

    pub fn get_abs_root_path(abs_path: Option<&String>, name: &str, req_index: usize) -> String {
        use std::string::ToString;

        let req_index = req_index.to_string();

        let mut built_path = String::new();
        match abs_path {
            Some(path) => built_path.push_str(path.as_str()),
            None => { },
        }
        built_path.push_str(name);
        built_path.push('/');
        built_path.push_str(req_index.as_str());
        built_path.push('/');

        built_path
    }

    pub fn make_requestable_uri(&self, link: &str) -> Uri {
        let link_uri = match Uri::from_str(link) {
            Ok(link) => link,
            Err(err) => panic!("ResultHandler::make_requestable_uri : {}", err),
        };

        if link_uri.is_absolute() {
            return link_uri;
        }

        let source_url = match self.get_extra_inform(ExtraInformKey::SourceUrl) {
            Some(source_url) => source_url,
            None => panic!("ResultHandler::make_requestable_uri - source url is none"),
        };
        let source_uri: Uri = source_url.parse().unwrap();

        let link_scheme = match link_uri.scheme() {
            Some(scheme) => scheme,
            None => {
                source_uri.scheme().unwrap()
            },
        };

        let link_host = match link_uri.host() {
            Some(host) => host,
            None => {
                source_uri.host().unwrap()
            },
        };

        let mut requestable_url = String::new();
        requestable_url.push_str(link_scheme);
        requestable_url.push_str("://");
        requestable_url.push_str(link_host);
        if link_uri.port() != None || source_uri.port() != None {
            let link_port = match link_uri.port() {
                Some(port) => port,
                None => {
                    source_uri.port().unwrap()
                },
            };
            requestable_url.push_str(":");
            requestable_url.push_str(link_port.to_string().as_str());
        }
        requestable_url.push_str(link);

        Uri::from_str(requestable_url.as_str()).unwrap()
    }
}

pub enum ReservedLabel {
    Part,
    Store,
    Collect,
    Link,
    Type,
}

impl FromStr for ReservedLabel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lowercase = s.to_lowercase();
        let lowercase = lowercase.as_str();
        let result = match lowercase {
            "part" => {
                Result::Ok(ReservedLabel::Part)
            },
            "store" => {
                Result::Ok(ReservedLabel::Store)
            },
            "collect" => {
                Result::Ok(ReservedLabel::Collect)
            },
            "link" => {
                Result::Ok(ReservedLabel::Link)
            },
            "type" => {
                Result::Ok(ReservedLabel::Type)
            },
            _ => Result::Err("ReservedLabel error".to_owned()),
        };
        result
    }
}

impl ToString for ReservedLabel {
    fn to_string(&self) -> String {
        let owned = match self {
            &ReservedLabel::Part => "part".to_owned(),
            &ReservedLabel::Store => "store".to_owned(),
            &ReservedLabel::Collect => "collect".to_owned(),
            &ReservedLabel::Link => "link".to_owned(),
            &ReservedLabel::Type => "type".to_owned(),
        };
        owned
    }
}