use hyper::Client;
use hyper::client::RedirectPolicy;
use hyper::client::response::Response;
use hyper::header::{Authorization, Basic, Bearer, ContentType, Headers, Location};
use hyper::mime::{Attr, Mime, TopLevel, SubLevel, Value};
use std::io::Read;

use super::{Auth, HttpClient, HttpRequest, HttpResponse, HttpStatus};
use super::datatype::Error;

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

    fn send_request(&mut self, req: &HttpRequest) -> Result<HttpResponse, Error> {
        debug!("send_request, request: {}", req.to_string());

        let mut headers  = Headers::new();
        let mut req_body = String::new();

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

                req_body.push_str("grant_type=client_credentials")

            }

            (Some(Auth::Token(token)), body) => {

               headers.set(Authorization(Bearer {
                   token: token.access_token.clone()
               }));

               if let Some(json) = body {

                   headers.set(ContentType(Mime(
                       TopLevel::Application,
                       SubLevel::Json,
                       vec![(Attr::Charset, Value::Utf8)])));

                   req_body.push_str(&json)
               }

            }

            (None, body) => {

               if let Some(json) = body {

                   headers.set(ContentType(Mime(
                       TopLevel::Application,
                       SubLevel::Json,
                       vec![(Attr::Charset, Value::Utf8)])));

                   req_body.push_str(&json)

               }

            }

            _ => {}

        }

        debug!("send_request, headers:  `{}`", headers);
        debug!("send_request, req_body: `{}`", req_body);

        let mut resp = try!(self.client
                            .request(req.method.clone().into_owned().into(),
                                     req.url.clone().into_owned())
                            .headers(headers)
                            .body(&req_body)
                            .send());

        if resp.status.is_success() {
            let mut data = Vec::new();
            let _: usize = try!(resp.read_to_end(&mut data));
            Ok(HttpResponse {
                status: HttpStatus::Ok,
                body:   data,
            })
        } else if resp.status.is_redirection() {
            let req = try!(relocate_request(req, &resp));
            self.send_request(&req)
        } else {
            let mut rbody = String::new();
            let _: usize = try!(resp.read_to_string(&mut rbody));
            Err(Error::ClientError(format!("Request errored with status {}, body: {}", resp.status, rbody)))
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
