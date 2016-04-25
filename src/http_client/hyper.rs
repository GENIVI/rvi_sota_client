use hyper::Client;
use hyper::client::RedirectPolicy;
use hyper::client::response::Response;
use hyper::header::{Authorization, Basic, Bearer, ContentType, Headers, Location};
use hyper::mime::{Attr, Mime, TopLevel, SubLevel, Value};
use rustc_serialize::json;
use std::fs::File;
use std::io::{Read, Write, BufReader, BufWriter};

use datatype::Error;
use http_client::{Auth, HttpClient, HttpRequest};


pub struct Hyper {
    client: Client,
}

impl Hyper {
    pub fn new() -> Hyper {
        let mut client = Client::new();
        client.set_redirect_policy(RedirectPolicy::FollowNone);
        Hyper { client: client }
    }
}

impl HttpClient for Hyper {

    fn send_request_to(&self, req: &HttpRequest, file: &mut File) -> Result<(), Error> {

        let mut headers = Headers::new();
        let mut body    = String::new();

        match (req.auth.clone().map(|a| a.into_owned()), req.body.to_owned()) {

            (None, None) => {}

            (Some(Auth::Credentials(ref id, ref secret)), None) => {

                headers.set(Authorization(Basic {
                    username: id.get.clone(),
                    password: Some(secret.get.clone())
                }));

                headers.set(ContentType(Mime(
                    TopLevel::Application,
                    SubLevel::WwwFormUrlEncoded,
                    vec![(Attr::Charset, Value::Utf8)])));

                body.push_str("grant_type=client_credentials")

            }

            (Some(Auth::Token(token)), Some(body)) => {

               headers.set(Authorization(Bearer {
                   token: token.access_token.clone()
               }));

               headers.set(ContentType(Mime(
                   TopLevel::Application,
                   SubLevel::Json,
                   vec![(Attr::Charset, Value::Utf8)])));

               let json: String = try!(json::encode(&body));

               body.into_owned().push_str(&json)

            }

            _ => panic!("hyper's send_request_to has been misused, this is a bug.")

        }

        let mut resp = try!(self.client
                            .request(req.method.clone().into_owned().into(),
                                     req.url.clone().into_owned())
                            .headers(headers)
                            .body(&body)
                            .send());

        if resp.status.is_success() {

            let mut rbody = String::new();
            let _: usize = try!(resp.read_to_string(&mut rbody));

            try!(tee(rbody.as_bytes(), file));
            Ok(())

        } else if resp.status.is_redirection() {
            let req = try!(relocate_request(req, &resp));
            self.send_request_to(&req, file)
        } else {
            Err(Error::ClientError(format!("Request errored with status {}", resp.status)))
        }

    }

}

fn relocate_request<'a>(req: &'a HttpRequest, resp: &Response) -> Result<HttpRequest<'a>, Error> {

    if let Some(&Location(ref loc)) = resp.headers.get::<Location>() {

        let url = try!(req.url.join(loc));

        Ok(HttpRequest {
            url:     url.into(),
            method:  req.method.clone(),
            auth:    None,
            body:    req.body.clone(),
        })

    } else {
        Err(Error::ClientError("Redirect with no Location header".to_string()))
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
