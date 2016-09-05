use chan;
use chan::{Sender, Receiver};

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

/// Return the body of an HTTP response on success, or an `Error` otherwise.
pub type Response = Result<Vec<u8>, Error>;
