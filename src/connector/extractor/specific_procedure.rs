use hyper::{Uri,Request,Method};
use hyper::header::Headers;
use result::{ExtraInformKey,ResultHandler};

pub trait SpecificProcedure {
    fn get_request(&self, link: &str) -> Request;
}

pub struct NaverWebtoonProcedure<'a> {
    handler: &'a ResultHandler,
}

impl<'a> NaverWebtoonProcedure<'a> {
    pub fn new(handler: &'a ResultHandler) -> NaverWebtoonProcedure {
        NaverWebtoonProcedure {
            handler: handler,
        }
    }

    fn add_domain_specific_headers(result_handler: &ResultHandler, headers: &mut Headers) {
        use connector::core::str::FromStr;

        let source_url = match result_handler.get_extra_inform(ExtraInformKey::SourceUrl) {
            Some(source_url) => source_url,
            None => {
                return;
            }
        };

        let parsed_source_url: Uri = match Uri::from_str(source_url.as_str()) {
            Ok(url) => url,
            Err(err) => {
                warn!("NaverWebtoonProcedure::add_domain_specific_headers - url parse error : {}", err);
                return;
            },
        };

        let domain = match parsed_source_url.host() {
            Some(domain) => domain,
            None => {
                warn!("NaverWebtoonProcedure::add_domain_specific_headers - url doesn't have host");
                return;
            },
        };

        if domain == "comic.naver.com" {
            let source_url_bytes = source_url.clone().into_bytes();
            headers.append_raw("Referer", source_url_bytes);
        } else {
            info!("NaverWebtoonProcedure::add_domain_specific_headers - domain is {}", domain);
        }
    }
}

impl<'a> SpecificProcedure for NaverWebtoonProcedure<'a> {
    fn get_request(&self, link: &str) -> Request {
        let link = self.handler.make_requestable_uri(link);
        let mut request = Request::new(Method::Get, link);
        match self.handler.get_raw_cookies() {
            Some(cookie) => {
                let mut req_headers = request.headers_mut();
                req_headers.set_raw("Set-Cookie", cookie);
                NaverWebtoonProcedure::add_domain_specific_headers(self.handler,
                                                                   &mut req_headers);
            },
            None => { },
        }
        request
    }
}

pub struct DefaultProcedure<'a> {
    handler: &'a ResultHandler,
}

impl<'a> DefaultProcedure<'a> {
    pub fn new(handler: &'a ResultHandler) -> DefaultProcedure {
        DefaultProcedure {
            handler: handler,
        }
    }
}

impl<'a> SpecificProcedure for DefaultProcedure<'a> {
    fn get_request(&self, link: &str) -> Request {
        let link = self.handler.make_requestable_uri(link);
        let mut request = Request::new(Method::Get, link);
        match self.handler.get_raw_cookies() {
            Some(cookie) => {
                let mut req_headers = request.headers_mut();
                req_headers.set_raw("Set-Cookie", cookie);
            },
            None => { },
        }
        request
    }
}