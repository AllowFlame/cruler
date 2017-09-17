use configure::RuleUtils;

#[test]
fn find_labels_test() {
    let pattern = "<img[ \\t\\r\\n\\v\\f]*src=[\"](?P<store>[0-9a-zA-Z:/\\._\\?=&]*)[\"]";
    let labels = RuleUtils::find_labels(pattern);
    let first_label = labels[0].as_str();
    assert_eq!("store", first_label);
}

#[test]
fn get_matched_test() {
    let content = "<img src=\"http://monolev.com/\" title=\"monolev test\"";
    let pattern = "<img[ \\t\\r\\n\\v\\f]*src=[\"](?P<store>[0-9a-zA-Z:/\\._\\?=&]*)[\"]";
    let label_name = "store";
    let matched = RuleUtils::get_matched(content, pattern, label_name);
    let first_matched = matched[0].as_str();
    assert_eq!("http://monolev.com/", first_matched);
}