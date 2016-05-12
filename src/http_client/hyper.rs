use hyper::Client;
use hyper::client::RedirectPolicy;
use hyper::client::response::Response;
use hyper::header::{Authorization, Basic, Bearer, ContentType, Headers, Location};
use hyper::mime::{Attr, Mime, TopLevel, SubLevel, Value};
use rustc_serialize::json;
use std::fs::File;
use std::io::{copy, Read};

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

    fn send_request_to(&mut self, req: &HttpRequest, file: &mut File) -> Result<(), Error> {

        debug!("send_request_to, request: {}", req.to_string());

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

            (Some(Auth::Token(token)), body) => {

               headers.set(Authorization(Bearer {
                   token: token.access_token.clone()
               }));

               if let Some(body) = body {

                   headers.set(ContentType(Mime(
                       TopLevel::Application,
                       SubLevel::Json,
                       vec![(Attr::Charset, Value::Utf8)])));

                   let json: String = try!(json::encode(&body));

                   body.into_owned().push_str(&json)

               }

            }

            _ => panic!("hyper's send_request_to has been misused, this is a bug.")

        }

        debug!("send_request_to, headers: `{}`", headers);
        debug!("send_request_to, body:    `{}`", body);

        let mut resp = try!(self.client
                            .request(req.method.clone().into_owned().into(),
                                     req.url.clone().into_owned())
                            .headers(headers)
                            .body(&body)
                            .send());

        if resp.status.is_success() {

            let mut rbody = String::new();
            let _: usize = try!(resp.read_to_string(&mut rbody));

            debug!("send_request_to, response: `{}`", rbody);
            debug!("send_request_to, file: `{:?}`", file);

            try!(copy(&mut rbody.as_bytes(), file));

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
            url:    url.into(),
            method: req.method.clone(),
            auth:   None,
            body:   req.body.clone(),
        })

    } else {
        Err(Error::ClientError("Redirect with no Location header".to_string()))
    }

}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::SeekFrom;
    use std::io::prelude::*;
    use tempfile;

    use super::*;
    use datatype::Url;
    use http_client::{Auth, HttpClient, HttpRequest};


    #[test]
    fn test_send_request_get() {

        let mut client: &mut HttpClient = &mut Hyper::new();

        let req = HttpRequest::get::<_, Auth>(
            Url::parse("https://eu.httpbin.org/get").unwrap(), None);

        let s: String = client.send_request(&req).unwrap();

        assert!(s != "".to_string())

    }

    #[test]
    fn test_send_request_to_get() {

        let mut client = &mut Hyper::new();

        let req = HttpRequest::get::<_, Auth>(
            Url::parse("https://eu.httpbin.org/get").unwrap(), None);

        let mut temp_file: File = tempfile::tempfile().unwrap();
        client.send_request_to(&req, &mut temp_file).unwrap();

        temp_file.seek(SeekFrom::Start(0)).unwrap();

        let mut buf = String::new();
        let _: usize = temp_file.read_to_string(&mut buf).unwrap();

        assert!(buf != "".to_string())
    }
}
