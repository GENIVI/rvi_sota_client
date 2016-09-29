use chan;
use chan::{Sender, Receiver};
use hyper::status::StatusCode;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str;

use datatype::{Error, Method, Url};


/// Abstracts a particular HTTP Client implementation with the basic methods
/// for sending `Request`s and receiving asynchronous `Response`s via a channel.
pub trait Client {
    fn chan_request(&self, req: Request, resp_tx: Sender<Response>);

    fn send_request(&self, req: Request) -> Receiver<Response> {
        let (resp_tx, resp_rx) = chan::async::<Response>();
        self.chan_request(req, resp_tx);
        resp_rx
    }

    fn get(&self, url: Url, body: Option<Vec<u8>>) -> Receiver<Response> {
        self.send_request(Request { method: Method::Get, url: url, body: body })
    }

    fn post(&self, url: Url, body: Option<Vec<u8>>) -> Receiver<Response> {
        self.send_request(Request { method: Method::Post, url: url, body: body })
    }

    fn put(&self, url: Url, body: Option<Vec<u8>>) -> Receiver<Response> {
        self.send_request(Request { method: Method::Put, url: url, body: body })
    }

    fn is_testing(&self) -> bool { false }
}


/// A simplified representation of an HTTP request for use in the client.
#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub url:    Url,
    pub body:   Option<Vec<u8>>
}


/// A Response enumerates between a successful (e.g. 2xx) HTTP response, a failed
/// (e.g. 4xx/5xx) response, or an Error before receiving any response.
#[derive(Debug)]
pub enum Response {
    Success(ResponseData),
    Failed(ResponseData),
    Error(Error)
}

impl Display for Response {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            Response::Success(ref data) => write!(f, "{}", data),
            Response::Failed(ref data)  => write!(f, "{}", data),
            Response::Error(ref err)    => write!(f, "{}", err),
        }
    }
}


/// Wraps the HTTP Status Code as well as any returned body.
#[derive(Debug)]
pub struct ResponseData {
    pub code: StatusCode,
    pub body: Vec<u8>
}

impl Display for ResponseData {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self.body.len() {
            0 => write!(f, "Response Code: {}", self.code),
            n => match str::from_utf8(&self.body) {
                Ok(text) => write!(f, "Response Code: {}, Body:\n{}", self.code, text),
                Err(_)   => write!(f, "Response Code: {}, Body: {} bytes", self.code, n),
            }
        }
    }
}
