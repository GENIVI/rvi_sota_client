use chan;
use chan::{Sender, Receiver};

use datatype::{Error, Method, Url};


pub trait Client {
    fn chan_request(&self, req: Request, resp_tx: Sender<Response>);

    fn send_request(&self, req: Request) -> Receiver<Response> {
        let (resp_tx, resp_rx) = chan::async::<Response>();
        self.chan_request(req, resp_tx);
        resp_rx
    }

    fn get(&self, url: Url, body: Option<Vec<u8>>) -> Receiver<Response> {
        self.send_request(Request::get(url, body))
    }

    fn post(&self, url: Url, body: Option<Vec<u8>>) -> Receiver<Response> {
        self.send_request(Request::post(url, body))
    }

    fn put(&self, url: Url, body: Option<Vec<u8>>) -> Receiver<Response> {
        self.send_request(Request::put(url, body))
    }

    fn is_testing(&self) -> bool { false }
}


#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub url:    Url,
    pub body:   Option<Vec<u8>>
}

impl Request {
    pub fn get(url: Url, body: Option<Vec<u8>>) -> Request {
        Request { method: Method::Get, url: url, body: body }
    }

    pub fn post(url: Url, body: Option<Vec<u8>>) -> Request {
        Request { method: Method::Post, url: url, body: body }
    }

    pub fn put(url: Url, body: Option<Vec<u8>>) -> Request {
        Request { method: Method::Put, url: url, body: body }
    }
}

pub type Response = Result<Vec<u8>, Error>;
