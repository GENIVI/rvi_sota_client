use hyper;
use hyper::method::Method;
use hyper::header::{Headers, Header, HeaderFormat};
use hyper::Url;
use error::Error;

use std::io::Read;


#[derive(Clone, Debug)]
pub struct HttpRequest<'a> {
    pub url: Url,
    pub method: Method,
    pub headers: Headers,
    pub body: Option<&'a str>
}

impl<'a> HttpRequest<'a> {
    pub fn new(url: Url, method: Method) -> HttpRequest<'a> {
        HttpRequest { url: url, method: method, headers: Headers::new(), body: None }
    }

    #[allow(dead_code)]
    pub fn get(url: Url) -> HttpRequest<'a> {
        HttpRequest::new(url, Method::Get)
    }

    pub fn post(url: Url) -> HttpRequest<'a> {
        HttpRequest::new(url, Method::Post)
    }

    pub fn with_body(&self, body: &'a str) -> HttpRequest<'a> {
        HttpRequest { body: Some(body), ..self.clone() }
    }

    pub fn with_header<H: Header + HeaderFormat>(&self, header: H) -> HttpRequest<'a> {
        let mut hs = self.headers.clone();
        hs.set(header);
        HttpRequest { headers: hs, ..self.clone() }
    }
}

pub trait HttpClient {
    fn new() -> Self;
    fn send_request(&self, req: &HttpRequest) -> Result<String, Error>;
}

impl HttpClient for hyper::Client {

    fn new() -> hyper::Client {
        hyper::Client::new()
    }

    fn send_request(&self, req: &HttpRequest) -> Result<String, Error> {
        self.request(req.method.clone(), req.url.clone())
            .headers(req.headers.clone())
            .body(if let Some(body) = req.body { body } else { "" })
            .send()
            .map_err(|e| {
                Error::ClientError(format!("Cannot send request: {}", e))
            })
            .and_then(|mut resp| {
                let mut rbody = String::new();
                resp.read_to_string(&mut rbody)
                    .map_err(|e| Error::ParseError(format!("Cannot read response: {}", e)))
                    .map(|_| rbody)
            })
    }
}
