use hyper::{Request,Method,Error};
use connector::{HeaderContentType,Connector};
use connector::futures::*;

#[test]
fn request_count_test() {
    let google_request = Request::new(Method::Get, "http://google.com".parse().unwrap());
    let naver_request = Request::new(Method::Get, "http://naver.com".parse().unwrap());
    let daum_request = Request::new(Method::Get, "http://daum.net".parse().unwrap());

    let mut conn = Connector::new();
    conn.add_request(google_request);
    conn.add_request(naver_request);
    let request_count = conn.get_requests_count();
    assert_eq!(2, request_count);

    conn.add_request(daum_request);
    let request_count = conn.get_requests_count();
    assert_eq!(3, request_count);

    conn.clear_requests();
    let request_count = conn.get_requests_count();
    assert_eq!(0, request_count);
}

#[test]
fn run_request_test() {
    let google_request = Request::new(Method::Get, "http://google.com".parse().unwrap());

    let mut conn = Connector::new();
    conn.add_request(google_request);

    let result = conn.run_request_all(|index, response| {
        assert_eq!(0, index);

        let content_type = Connector::get_content_type(&response);
        let is_matched = match content_type {
            HeaderContentType::Text(content_type) => true,
            _ => false,
        };
        assert_eq!(true, is_matched);

        response.body().fold(Vec::new(), |mut v, chunk| {
            v.extend(&chunk[..]);
            future::ok::<_, Error>(v)
        }).and_then(|chunks| {
            let body_content = String::from_utf8(chunks).unwrap();
            future::ok(body_content)
        })
    });

    match result {
        Ok(content) => {
            assert!(true);
        },
        Err(err) => {
            assert!(false);
        },
    }
}