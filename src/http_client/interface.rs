use hyper::header::{Headers, Header, HeaderFormat, Location, Authorization, Bearer};
use hyper::method::Method;
use hyper::client::response::Response;
use hyper;
use std::io::{Read, Write, BufReader, BufWriter};

use datatype::{Error, Url};


#[derive(Clone, Debug)]
pub struct HttpRequest<'a> {
    pub url: Url,
    pub method: Method,
    pub headers: Headers,
    pub body: Option<&'a str>,
}

impl<'a> ToString for HttpRequest<'a> {
    fn to_string(&self) -> String {
        format!("{} {}", self.method, self.url.to_string())
    }
}

impl<'a> HttpRequest<'a> {
    pub fn new(url: Url, method: Method) -> HttpRequest<'a> {
        HttpRequest {
            url: url,
            method: method,
            headers: Headers::new(),
            body: None,
        }
    }

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
    fn send_request_to<W: Write>(&self, req: &HttpRequest, to: W) -> Result<(), Error>;
}

impl HttpClient for hyper::Client {
    fn new() -> hyper::Client {
        let mut client = hyper::Client::new();
        client.set_redirect_policy(hyper::client::RedirectPolicy::FollowNone);
        client
    }

    fn send_request(&self, req: &HttpRequest) -> Result<String, Error> {
        self.request(req.method.clone(), req.url.clone())
            .headers(req.headers.clone())
            .body(req.body.unwrap_or(""))
            .send()
            .map_err(|e| Error::ClientError(format!("{}", e)))
            .and_then(|mut resp| {
                match resp.status.class() {
                    hyper::status::StatusClass::Success => {
                        let mut rbody = String::new();
                        resp.read_to_string(&mut rbody)
                            .map_err(|e| Error::ParseError(format!("Cannot read response: {}", e)))
                            .map(|_| rbody)
                    }
                    hyper::status::StatusClass::Redirection => {
                        relocate_request(req, &resp).and_then(|ref r| self.send_request(r))
                    }
                    _ => {
                        Err(Error::ClientError(format!("Request failed with status {}",
                                                       resp.status)))
                    }
                }
            })
    }

    fn send_request_to<W: Write>(&self, req: &HttpRequest, to: W) -> Result<(), Error> {
        self.request(req.method.clone(), req.url.clone())
            .headers(req.headers.clone())
            .body(req.body.unwrap_or(""))
            .send()
            .map_err(|e| Error::ClientError(format!("{}", e)))
            .and_then(|resp| {
                match resp.status.class() {
                    hyper::status::StatusClass::Success => {
                        tee(resp, to)
                            .map_err(|e| Error::ParseError(format!("Cannot read response: {}", e)))
                            .map(|_| ())
                    }
                    hyper::status::StatusClass::Redirection => {
                        relocate_request(req, &resp).and_then(|ref r| self.send_request_to(r, to))
                    }
                    _ => {
                        Err(Error::ClientError(format!("Request failed with status {}",
                                                       resp.status)))
                    }
                }
            })
    }
}

fn relocate_request<'a>(req: &'a HttpRequest, resp: &Response) -> Result<HttpRequest<'a>, Error> {
    match resp.headers.get::<Location>() {
        Some(&Location(ref loc)) => {
            req.url
               .join(loc)
               .map_err(|e| Error::ParseError(format!("Cannot read location: {}", e)))
               .and_then(|url| {
                   let mut headers = req.headers.clone();
                   headers.remove::<Authorization<Bearer>>();
                   Ok(HttpRequest {
                       url: url,
                       method: req.method.clone(),
                       headers: headers,
                       body: req.body,
                   })
               })
        }
        None => Err(Error::ClientError("Redirect with no Location header".to_string())),
    }
}

pub fn tee<R: Read, W: Write>(from: R, to: W) -> Result<(), Error> {
    let mb = 1024 * 1024;
    let rbuf = BufReader::with_capacity(5 * mb, from);
    let mut wbuf = BufWriter::with_capacity(5 * mb, to);
    for b in rbuf.bytes() {
        try!(wbuf.write(&[try!(b)]));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{Read, repeat};

    #[test]
    fn test_tee() {
        let values = repeat(b'a').take(9000);
        let sink = File::create("/tmp/otaplus_tee_test").unwrap();

        assert!(tee(values, sink).is_ok());

        let mut values2 = repeat(b'a').take(9000);
        let mut expected = Vec::new();
        let _ = values2.read_to_end(&mut expected);

        let mut f = File::open("/tmp/otaplus_tee_test").unwrap();
        let mut result = Vec::new();
        let _ = f.read_to_end(&mut result);

        assert_eq!(result, expected);
    }

}
