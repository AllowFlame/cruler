extern crate futures;
extern crate tokio_core;
extern crate core;
extern crate hyper;
extern crate hyper_tls;

pub mod extractor;
pub mod navigator;
pub mod connector_utils;
#[cfg(test)]
mod connector_test;

use std::cell::{Ref,RefCell};
use std::vec::Vec;
use std::collections::VecDeque;

use hyper::{Client,Request,Body,Uri};
use hyper::client::{HttpConnector,Response};
use self::hyper_tls::HttpsConnector;
use hyper::header::{Raw};

use self::futures::future::{IntoFuture};
use self::tokio_core::*;

enum HeaderContentType {
    Image(String),
    Text(String),
    Others(String),
}

impl HeaderContentType {
    //FIXME: standard from_str implementation would be necessary
    pub fn from_str(content_type: &str) -> HeaderContentType {
        if content_type.starts_with("image") {
            let image_format = HeaderContentType::get_image_format(content_type);
            HeaderContentType::Image(image_format.to_owned())
        }
        else if content_type.starts_with("text") {
            let text_format = HeaderContentType::get_text_format(content_type);
            HeaderContentType::Text(text_format.to_owned())
        }
        else {
            HeaderContentType::Others(content_type.to_owned())
        }
    }

    fn get_image_format<'a>(content_type: &'a str) -> &'a str {
        //NOTE: image/jpeg, image/png, Application/... can be received
        let split_collection: Vec<&str> = content_type.split("/").collect();
        return split_collection[1];
    }

    fn get_text_format<'a>(content_type: &'a str) -> &'a str {
        //NOTE: text/html, text/xml, text/css, Application/xml... can be received
        let split_collection: Vec<&str> = content_type.split("/").collect();
        return split_collection[1];
    }
}


//pub trait ResponseHandler {
//    fn response_callback(&self, index: usize, res: Response) -> Stream<Item=hyper::Chunk,Error=hyper::Error>;
//}

pub struct Connector {
    core: RefCell<reactor::Core>,
    client: RefCell<Client<HttpsConnector<HttpConnector>,Body>>,
    requests: RefCell<VecDeque<Request>>,
}

impl Connector {
    pub fn new() -> Connector {
        let core = reactor::Core::new().unwrap();
        let client = Client::configure().
            connector(HttpsConnector::new(20, &core.handle()).unwrap()).
            build(&core.handle());
        Connector {
            core: RefCell::new(core),
            client: RefCell::new(client),
            requests: RefCell::new(VecDeque::new()),
        }
    }

    pub fn get_requests_count(&self) -> usize {
        let length = self.requests.borrow().len();
        length
    }

    pub fn add_request(&mut self, req: Request) {
        self.requests.borrow_mut().push_back(req);
    }

    pub fn add_requests(&mut self, reqs: &mut VecDeque<Request>) {
        self.requests.borrow_mut().append(reqs);
    }

    pub fn clear_requests(&mut self) {
        self.requests.borrow_mut().clear();
    }

    pub fn requests(&self) -> Ref<VecDeque<Request>> {
        self.requests.borrow()
    }

    pub fn request_urls(&self) -> Vec<String> {
        let mut urls = Vec::new();

        let requests = self.requests();
        for request in requests.iter() {
            let uri: &Uri = request.uri();
            let url = uri.as_ref();
            urls.push(url.to_owned());
        }
        urls
    }

    pub fn run_request_all<F, B>(&mut self, f: F) -> Result<Vec<B::Item>, hyper::Error>
        where
            F: Fn(usize, Response) -> B,
            B: IntoFuture<Error=::hyper::Error> {
        use self::futures::future::*;

        let function = &f;
        let mut req_futures = Vec::new();
        let mut requests = self.requests.borrow_mut();
        let mut pass_index: usize = 0;
        loop {
            let pop = requests.pop_front();
            match pop {
                Option::None => break,
                Option::Some(request) => {
                    let client = self.client.borrow_mut();
                    let job = client.request(request).and_then(move |res| {
                        function(pass_index, res)
                    });
                    req_futures.push(job);
                },
            }
            pass_index += 1;
        }

        let work = futures::future::join_all(req_futures);

        let mut core = self.core.borrow_mut();
        core.run(work)
    }


    pub fn get_header_raw_value(response: &Response, key_name: &str) -> Option<Raw> {
        let headers = response.headers();
        let raw_value: &Raw = match headers.get_raw(key_name) {
            Option::Some(value) => value,
            Option::None => {
                info!("Connector::get_header_raw_value - Header doesn't have values of {}", key_name);
                return Option::None;
            },
        };

        let owned = raw_value.clone();
        Some(owned)
    }

    fn get_raw_cookies(response: &Response) -> Option<Raw> {
        Connector::get_header_raw_value(response, "Set-Cookie")
    }


    fn get_content_type(response: &Response) -> HeaderContentType {
        let raw_value = Connector::get_header_raw_value(response, "content-type");

        let content_type: Raw = match raw_value {
            Option::Some(content) => content,
            Option::None => {
                return HeaderContentType::Others("no content-type".to_owned());
            }
        };

        if content_type.len() > 1 {
            return HeaderContentType::Others("content-type is more than 2".to_string());
        }

        for value in content_type.iter() {
            let content = String::from_utf8(value.to_vec()).expect("Found invalid UTF-8");

            let header_content_type = HeaderContentType::from_str(content.as_str());
            return header_content_type;
        }

        return HeaderContentType::Others("unknown".to_owned());
    }


}
