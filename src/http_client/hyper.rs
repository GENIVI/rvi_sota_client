use hyper::Client;
use hyper::header::{Authorization, Bearer, ContentType, Headers};
use hyper::mime::{Attr, Mime, TopLevel, SubLevel, Value};
use std::io::{Read, Write, BufReader, BufWriter};

use datatype::Error;
use http_client::{HttpClient2, HttpRequest2};


pub struct Hyper {
    client: Client,
}

impl HttpClient2 for Hyper {

    fn send_request_to<T: Read + Write>
        (&self, request: &HttpRequest2, target: T) -> Result<(), Error> {

        let mut headers = Headers::new();

        if let Some(token) = request.token {
            headers.set(Authorization(Bearer {
                token: token.access_token.clone()
            }))
        }

        if request.body.is_some() {
            headers.set(ContentType(Mime(
                TopLevel::Application,
                SubLevel::Json,
                vec![(Attr::Charset, Value::Utf8)])))
        }

        let mut resp = try!(self.client
                            .request(request.method.into(), request.url.clone())
                            .headers(headers)
                            .body(
                                if let Some(body) = request.body
                                                           .and_then(|b| b.as_string()) {
                                    body
                                } else {
                                    ""
                                })
                            .send());

        let mut rbody = String::new();
        let status    = resp.status;

        if status.is_server_error() || status.is_client_error() {
            Err(Error::ClientError(format!("Request errored with status {}", status)))
        } else {
            tee(resp, target);
            Ok(())
        }

    }

}

pub fn tee<R: Read, W: Write>(from: R, to: W) -> Result<(), Error> {

    const BUF_SIZE: usize = 1024 * 1024 * 5;

    let     rbuf = BufReader::with_capacity(BUF_SIZE, from);
    let mut wbuf = BufWriter::with_capacity(BUF_SIZE, to);

    for b in rbuf.bytes() {
        try!(wbuf.write(&[try!(b)]));
    }

    Ok(())

}


#[cfg(test)]
mod tests {

    use std::fs::File;
    use std::io::{Read, repeat};

    use super::*;


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
