#[cfg(test)]
mod configure_test;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::collections::BTreeMap;
use std::str::FromStr;

use super::hyper::header::{Raw};
use super::regex;
use super::toml;

use result::*;

pub struct ConfigureError {
    msg: String,
}

impl ConfigureError {
    pub fn new(msg: &str) -> ConfigureError {
        ConfigureError {
            msg: msg.to_owned(),
        }
    }

    pub fn msg(&self) -> &str {
        self.msg.as_ref()
    }
}

pub trait RuleConfigure {
    fn parts(&self) -> Option<&Vec<String>>;
    fn extract(&self) -> Option<&String>;
}

pub struct RuleUtils {}

impl RuleUtils {
    pub fn find_labels(pattern: &str) -> Vec<String> {
        use self::regex::Regex;

        let mut matched = Vec::new();
        lazy_static! {
        static ref RE: Regex = Regex::new(r"\(\?P<([a-zA-Z_]*)>").unwrap();
        }
        for capture in RE.captures_iter(pattern) {
            let found = capture.get(1).unwrap().as_str();
            matched.push(found.to_owned());
        }

        matched
    }

    pub fn get_matched(content: &str, pattern: &str, label_name: &str) -> Vec<String> {
        use self::regex::Regex;

        let mut matched = Vec::new();
        let regex = Regex::new(pattern).unwrap();
        for capture in regex.captures_iter(content) {
            let label = match capture.name(label_name) {
                Some(label) => label.as_str(),
                None => continue,
            };
            matched.push(label.to_owned());
        }
        matched
    }

    pub fn read_file_content(file_path: &str) -> String {
        let path = Path::new(file_path);
        let display = path.display();

        let mut file = match File::open(&path) {
            Err(_) => panic!("couldn't open {}", display),
            Ok(file) => file,
        };

        let mut file_content = String::new();
        match file.read_to_string(&mut file_content) {
            Err(_) => panic!("couldn't read {}", display),
            Ok(_) => info!("{} has been read", display),
        }

        file_content
    }

    pub fn make_part_contents(content: String, parts_rule: &Vec<String>) -> Vec<String> {
        let mut part_contents: Vec<String> = Vec::new();
        part_contents.push(content);

        for part_rule in parts_rule {
            let mut label_part_map: BTreeMap<String, Vec<String>> = BTreeMap::new();
            let label_names: Vec<String> = RuleUtils::find_labels(part_rule.as_str());
            for part_content in &part_contents {
                for label_name in &label_names {
                    let part_matched: Vec<String> =
                        RuleUtils::get_matched(part_content.as_str(),
                                               part_rule.as_str(),
                                               label_name.as_str());
                    label_part_map.insert(label_name.clone(), part_matched);
                }
            }

            part_contents.clear();
            for (label_name, matched_contents) in label_part_map {
                if label_name != ReservedLabel::Part.to_string() {
                    continue;
                }

                for matched_content in matched_contents {
                    part_contents.push(matched_content);
                }
            }
        }

        part_contents
    }

    pub fn make_result_handlers(part_contents: Vec<String>, extract_rule: &String,
                            root_path: Option<String>, source_url: String,
                            raw_cookies: Option<Raw>) -> Vec<ResultHandler> {
        let mut label_extract_vec: Vec<ResultHandler> = Vec::new();
        let label_names: Vec<String> = RuleUtils::find_labels(extract_rule.as_str());
        for part_content in &part_contents {
            let mut result_handler = ResultHandler::new(root_path.clone(),
                                                        raw_cookies.clone());
            for label_name in &label_names {
                let extract_matched: Vec<String> =
                    RuleUtils::get_matched(part_content.as_str(),
                                           extract_rule.as_str(),
                                           label_name.as_str());

                result_handler.insert_result(label_name.as_str(), extract_matched);
                result_handler.insert_extra_inform(ExtraInformKey::SourceUrl, source_url.clone());
            }
            label_extract_vec.push(result_handler);
        }

        label_extract_vec
    }

}

#[derive(Deserialize)]
pub struct Configure {
    navigator: Option<NavigatorConfigure>,
    extractor: Option<ExtractorConfigure>,
    result: Option<ResultConfigure>,
}

impl Configure {
    pub fn new(file_path: &str) -> Configure {
        let content = RuleUtils::read_file_content(file_path);
        let content = content.as_str();

        let config: Configure = toml::from_str(content).unwrap();
        config
    }

    pub fn get_navigator_configure(&self) -> Option<&NavigatorConfigure> {
        self.navigator.as_ref()
    }

    pub fn get_extractor_configure(&self) -> Option<&ExtractorConfigure> {
        self.extractor.as_ref()
    }

    pub fn get_result_configure(&self) -> Option<&ResultConfigure> {
        self.result.as_ref()
    }
}

impl FromStr for Configure {
    type Err = ConfigureError;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let rules = match toml::from_str(content) {
            Result::Ok(rules) => Result::Ok(rules),
            Result::Err(_err) => Result::Err(ConfigureError::new("Configure::from_str error")),
        };
        rules
    }
}

impl Default for Configure {
    fn default() -> Configure {
        Configure::new("pack/configure.toml")
    }
}

#[derive(Deserialize)]
pub struct NavigatorConfigure {

}

#[derive(Deserialize)]
pub struct ExtractorConfigure {
    connection_pool_size: Option<i64>,
}

impl ExtractorConfigure {
    pub fn get_connection_pool_size(&self) -> Option<i64> {
        self.connection_pool_size
    }
}

#[derive(Deserialize)]
pub struct ResultConfigure {

}