use hyper;
use hyper::method::Method;
use hyper::header::{Headers, Header, HeaderFormat};
use hyper::Url;
use error::Error;

use std::io::{Read, Write, BufReader, BufWriter};


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
                let status = resp.status;
                if status.is_server_error() || status.is_client_error() {
                    Err(Error::ClientError(format!("Request errored with status {}", status)))
                } else {
                    resp.read_to_string(&mut rbody)
                        .map_err(|e| Error::ParseError(format!("Cannot read response: {}", e)))
                        .map(|_| rbody)
                }
            })
    }

    fn send_request_to<W: Write>(&self, req: &HttpRequest, to: W) -> Result<(), Error> {
        self.request(req.method.clone(), req.url.clone())
            .headers(req.headers.clone())
            .body(if let Some(body) = req.body { body } else { "" })
            .send()
            .map_err(|e| {
                Error::ClientError(format!("Cannot send request: {}", e))
            })
            .and_then(|resp| {
                let status = resp.status;
                if status.is_server_error() || status.is_client_error() {
                    Err(Error::ClientError(format!("Request errored with status {}", status)))
                } else {
                    tee(resp, to)
                        .map_err(|e| Error::ParseError(format!("Cannot read response: {}", e)))
                        .map(|_| ())
                }
            })
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
