use chan::Sender;
use hyper;
use hyper::{Encoder, Decoder, Next};
use hyper::client::{Client, Handler, HttpsConnector, Request, Response};
use hyper::header::{Authorization, Basic, Bearer, ContentLength, ContentType, Location};
use hyper::mime::{Attr, Mime, TopLevel, SubLevel, Value};
use hyper::net::{HttpStream, HttpsStream, OpensslStream, Openssl};
use hyper::status::StatusCode;
use std::{io, mem};
use std::io::{ErrorKind, Write};
use std::time::Duration;
use time;

use datatype::{Auth, Error};
use http_client::{HttpClient, HttpRequest, HttpResponse};


#[derive(Clone)]
pub struct AuthClient {
    auth:   Auth,
    client: Client<AuthHandler>,
}

impl AuthClient {
    pub fn new(auth: Auth) -> AuthClient {
        let client = Client::<AuthHandler>::configure()
            .keep_alive(true)
            .max_sockets(1024)
            .connector(HttpsConnector::new(Openssl::default()))
            .build()
            .expect("unable to create a new hyper Client");

        AuthClient {
            auth:   auth,
            client: client,
        }
    }
}

impl HttpClient for AuthClient {
    fn chan_request(&self, req: HttpRequest, resp_tx: Sender<HttpResponse>) {
        debug!("send_request_to: {:?}", req.url);
        let _ = self.client.request(req.url.inner(), AuthHandler {
            auth:     self.auth.clone(),
            req:      req,
            timeout:  Duration::from_secs(20),
            started:  None,
            written:  0,
            response: Vec::new(),
            resp_tx:  resp_tx.clone(),
        }).map_err(|err| resp_tx.send(Err(Error::from(err))));
    }
}


// FIXME: uncomment when yocto is at 1.8.0: #[derive(Debug)]
pub struct AuthHandler {
    auth:     Auth,
    req:      HttpRequest,
    timeout:  Duration,
    started:  Option<u64>,
    written:  usize,
    response: Vec<u8>,
    resp_tx:  Sender<HttpResponse>,
}

// FIXME: required for building on 1.7.0 only
impl ::std::fmt::Debug for AuthHandler {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "unimplemented")
    }
}

impl AuthHandler {
    fn redirect_request(&self, resp: Response) {
        info!("redirect_request");
        match resp.headers().get::<Location>() {
            Some(&Location(ref loc)) => match self.req.url.join(loc) {
                Ok(url) => {
                    debug!("redirecting to {:?}", url);
                    // drop Authentication Header on redirect
                    let client = AuthClient::new(Auth::None);
                    let body   = match self.req.body {
                        Some(ref data) => Some(data.clone()),
                        None           => None
                    };
                    let resp_rx = client.send_request(HttpRequest {
                        url:    url,
                        method: self.req.method.clone(),
                        body:   body,
                    });
                    match resp_rx.recv() {
                        Some(resp) => match resp {
                            Ok(data) => self.resp_tx.send(Ok(data)),
                            Err(err) => self.resp_tx.send(Err(Error::from(err)))
                        },
                        None       => panic!("no redirect_request response")
                    }
                }

                Err(err) => self.resp_tx.send(Err(Error::from(err)))
            },

            None => {
                let msg = "redirection without Location header".to_string();
                error!("{}", msg);
                self.resp_tx.send(Err(Error::ClientError(msg)))
            }
        }
    }
}

pub type Stream = HttpsStream<OpensslStream<HttpStream>>;

impl Handler<Stream> for AuthHandler {
    fn on_request(&mut self, req: &mut Request) -> Next {
        info!("on_request: {} {}", req.method(), req.uri());
        self.started = Some(time::precise_time_ns());

        req.set_method(self.req.method.clone().into());
        let mut headers = req.headers_mut();

        match self.auth {
            Auth::None => {
                headers.set(ContentType(Mime(TopLevel::Application, SubLevel::Json,
                                             vec![(Attr::Charset, Value::Utf8)])));
            }

            Auth::Credentials(_, _) if self.req.body.is_some() => {
                panic!("no request body expected for Auth::Credentials");
            }

            Auth::Credentials(ref id, ref secret) => {
                headers.set(Authorization(Basic { username: id.0.clone(),
                                                  password: Some(secret.0.clone()) }));
                headers.set(ContentType(Mime(TopLevel::Application, SubLevel::WwwFormUrlEncoded,
                                             vec![(Attr::Charset, Value::Utf8)])));
                self.req.body = Some(br#"grant_type=client_credentials"#.to_vec());
            }

            Auth::Token(ref token) => {
                headers.set(Authorization(Bearer { token: token.access_token.clone() }));
                headers.set(ContentType(Mime(TopLevel::Application, SubLevel::Json,
                                             vec![(Attr::Charset, Value::Utf8)])));
            }
        };

        match self.req.body {
            Some(ref body) => {
                headers.set(ContentLength(body.len() as u64));
                Next::write()
            }

            None => Next::read().timeout(self.timeout)
        }
    }

    fn on_request_writable(&mut self, encoder: &mut Encoder<Stream>) -> Next {
        let body = self.req.body.as_ref().expect("on_request_writable expects a body");

        match encoder.write(&body[self.written..]) {
            Ok(0) => {
                debug!("{} bytes written to request body", self.written);
                Next::read().timeout(self.timeout)
            },

            Ok(n) => {
                self.written += n;
                trace!("{} bytes written to request body", n);
                Next::write()
            }

            Err(ref err) if err.kind() == ErrorKind::WouldBlock => {
                trace!("retry on_request_writable");
                Next::write()
            }

            Err(err) => {
                error!("unable to write request body: {}", err);
                self.resp_tx.send(Err(Error::from(err)));
                Next::remove()
            }
        }
    }

    fn on_response(&mut self, resp: Response) -> Next {
        info!("on_response: status: {}, headers:\n{}", resp.status(), resp.headers());
        if let Some(started) = self.started {
            debug!("latency: {}", time::precise_time_ns() - started);
        }

        if resp.status().is_success() {
            if let Some(len) = resp.headers().get::<ContentLength>() {
                if **len > 0 {
                    return Next::read();
                }
            }
            self.resp_tx.send(Ok(Vec::new()));
            Next::end()
        } else if resp.status().is_redirection() {
            self.redirect_request(resp);
            Next::end()
        } else if resp.status() == &StatusCode::Forbidden {
            error!("on_response: 403 Forbidden");
            self.resp_tx.send(Err(Error::AuthorizationError("403".to_string())));
            Next::end()
        } else {
            let msg = format!("failed response status: {}", resp.status());
            error!("{}", msg);
            self.resp_tx.send(Err(Error::ClientError(msg)));
            Next::end()
        }
    }

    fn on_response_readable(&mut self, decoder: &mut Decoder<Stream>) -> Next {
        match io::copy(decoder, &mut self.response) {
            Ok(0) => {
                debug!("on_response_readable bytes read: {:?}", self.response.len());
                self.resp_tx.send(Ok(mem::replace(&mut self.response, Vec::new())));
                Next::end()
            }

            Ok(n) => {
                trace!("{} more response bytes read", n);
                Next::read()
            }

            Err(ref err) if err.kind() == ErrorKind::WouldBlock => {
                trace!("retry on_response_readable");
                Next::read()
            }

            Err(err) => {
                error!("unable to read response body: {}", err);
                self.resp_tx.send(Err(Error::from(err)));
                Next::end()
            }
        }
    }

    fn on_error(&mut self, err: hyper::Error) -> Next {
        error!("on_error: {}", err);
        self.resp_tx.send(Err(Error::from(err)));
        Next::remove()
    }
}


#[cfg(test)]
mod tests {
    use rustc_serialize::json::Json;

    use super::*;
    use datatype::{Auth, Method, Url};
    use http_client::{HttpClient, HttpRequest};


    #[test]
    fn test_send_get_request() {
        let client = AuthClient::new(Auth::None);
        let req = HttpRequest {
            method: Method::Get,
            url:    Url::parse("http://eu.httpbin.org/bytes/16?seed=123").unwrap(),
            body:   None,
        };

        let resp_rx = client.send_request(req);
        let data    = resp_rx.recv().unwrap().unwrap();
        assert_eq!(data, vec![13, 22, 104, 27, 230, 9, 137, 85, 218, 40, 86, 85, 62, 0, 111, 22]);
    }

    #[test]
    fn test_send_post_request() {
        let client = AuthClient::new(Auth::None);
        let req = HttpRequest {
            method: Method::Post,
            url:    Url::parse("https://eu.httpbin.org/post").unwrap(),
            body:   Some(br#"foo"#.to_vec()),
        };

        let resp_rx = client.send_request(req);
        let body    = resp_rx.recv().unwrap().unwrap();
        let resp    = String::from_utf8(body).unwrap();
        let json    = Json::from_str(&resp).unwrap();
        let obj     = json.as_object().unwrap();
        let data    = obj.get("data").unwrap().as_string().unwrap();
        assert_eq!(data, "foo");
    }
}
