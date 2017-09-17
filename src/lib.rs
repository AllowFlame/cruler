#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate hyper;
extern crate regex;
extern crate toml;
extern crate libc;

pub mod configure;
pub mod connector;
pub mod result;

use std::str::FromStr;

use std::ffi::CStr;
use std::os::raw::c_char;

#[no_mangle]
pub extern fn cruler_extract_all_with_default_config() {
    use configure::Configure;
    use connector::extractor::extraction_rules::ExtractionRules;
    use connector::extractor::Extractor;

    let ext_rules = ExtractionRules::default();
    let configure = Configure::default();
    let ext_configure = configure.get_extractor_configure().unwrap();

    let extractor = Extractor::new(&ext_rules, ext_configure);
    extractor.extract_all();
}

#[no_mangle]
pub extern fn cruler_extract_all_from_raw(ext_rule_raw: *const c_char,
                                             config_raw: *const c_char) {
    use configure::Configure;
    use connector::extractor::extraction_rules::ExtractionRules;
    use connector::extractor::Extractor;

    let ext_rules: ExtractionRules = unsafe {
        let input_c_str = CStr::from_ptr(ext_rule_raw);
        let ext_rule_str = match input_c_str.to_str() {
            Result::Ok(input) => input,
            Result::Err(_err) => {
                error!("cruler_extract_all_from_raw - input str error");
                return;
            }
        };

        let ext_rules = match ExtractionRules::from_str(ext_rule_str) {
            Result::Ok(rules) => rules,
            Result::Err(_err) => {
                error!("cruler_extract_all_from_raw - input str format error");
                return;
            }
        };
        ext_rules
    };

    let configure: Configure = unsafe {
        let input_c_str = CStr::from_ptr(config_raw);
        let config_str = match input_c_str.to_str() {
            Result::Ok(input) => input,
            Result::Err(_err) => {
                error!("");
                return;
            }
        };

        let configure = match Configure::from_str(config_str) {
            Result::Ok(configure) => configure,
            Result::Err(_err) => {
                error!("");
                return;
            }
        };
        configure
    };
    let ext_configure = configure.get_extractor_configure().unwrap();

    let extractor = Extractor::new(&ext_rules, ext_configure);
    extractor.extract_all();
}

#[no_mangle]
pub extern fn cruler_extract_all(config_path: *const c_char) {
    use connector::extractor::extraction_rules::ExtractionRules;

    let config_root_path = unsafe {
        let root_path = CStr::from_ptr(config_path);
        root_path.to_owned()
    };
    let root_path = match config_root_path.to_str() {
        Result::Ok(root_path) => root_path,
        Result::Err(_err) => {
            error!("cruler_extract_all - root_path error");
            return;
        }
    };

    let config_file_path = get_file_path(root_path, "configure.toml");
    let extract_file_path = get_file_path(root_path, "extraction_rules.toml");

    let ext_rules = ExtractionRules::new(extract_file_path.as_str());
    extractor_extract_all(&ext_rules, config_file_path.as_str());
}

#[inline(always)]
fn get_file_path(root_path: &str, file_name: &str) -> String {
    let mut file_path = String::new();
    file_path.push_str(root_path);
    file_path.push_str(file_name);
    file_path
}

#[inline(always)]
fn extractor_extract_all(ext_rules: &connector::extractor::extraction_rules::ExtractionRules,
                         config_path: &str) {
    use configure::Configure;
    use connector::extractor::Extractor;

    let configure = Configure::new(config_path);
    let ext_configure = configure.get_extractor_configure().unwrap();

    let extractor = Extractor::new(&ext_rules, ext_configure);
    extractor.extract_all();
}