use result::{ResultHandler,ExtraInformKey};

#[test]
fn key_test() {
    let source_url = ExtraInformKey::SourceUrl;
    assert_eq!("SourceUrl", source_url.to_string().as_str());
}

#[test]
fn get_abs_root_path_test() {
    use std::str::FromStr;

    let abs_path_input = Option::None;
    let name = "name";
    let req_index = 0;

    let rel_path = ResultHandler::get_abs_root_path(abs_path_input, name, req_index);
    assert_eq!("name/0/", rel_path.as_str());

    let abs_path_input = String::from("/root/path/");
    let abs_path = ResultHandler::get_abs_root_path(Some(&abs_path_input), name, req_index);
    assert_eq!("/root/path/name/0/", abs_path.as_str());
}