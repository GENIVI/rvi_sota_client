use hyper::Client;
use hyper::header::{Authorization, Basic, Bearer, ContentType, Headers};
use hyper::mime::{Attr, Mime, TopLevel, SubLevel, Value};
use rustc_serialize::json;
use std::fs::File;
use std::io::{Read, Write, BufReader, BufWriter};

use datatype::Error;
use http_client::{Auth, HttpClient2, HttpRequest2};


pub struct Hyper {
    client: Client,
}

impl HttpClient2 for Hyper {

    fn send_request_to(&self, request: &HttpRequest2, file: &mut File) -> Result<(), Error> {

        let mut headers = Headers::new();
        let mut body    = String::new();

        match *request.auth {
            Auth::Credentials(ref id, ref secret) =>
                headers.set(Authorization(Basic {
                    username: id.get.clone(),
                    password: Some(secret.get.clone())
                })),
            Auth::Token(token) =>
                headers.set(Authorization(Bearer {
                    token: token.access_token.clone()
                }))
        }

        if request.auth.is_credentials() && request.body.is_none() {

            headers.set(ContentType(Mime(
                TopLevel::Application,
                SubLevel::WwwFormUrlEncoded,
                vec![(Attr::Charset, Value::Utf8)])));

            body.push_str("grant_type=client_credentials")

        } else if request.auth.is_token() && request.body.is_some() {

            headers.set(ContentType(Mime(
                TopLevel::Application,
                SubLevel::Json,
                vec![(Attr::Charset, Value::Utf8)])));

            let json = try!(json::encode(&request.body.unwrap()));

            body.push_str(&json)

        } else {
            panic!("send_request_to has been misused, this is a bug.")
        }

        let mut resp = try!(self.client
                            .request(request.method.clone().into(), request.url.clone())
                            .headers(headers)
                            .body(&body)
                            .send());

        let status = resp.status;

        if status.is_server_error() || status.is_client_error() {
            Err(Error::ClientError(format!("Request errored with status {}", status)))
        } else {

            let mut rbody = String::new();
            let _: usize = try!(resp.read_to_string(&mut rbody));

            try!(tee(rbody.as_bytes(), file));
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
