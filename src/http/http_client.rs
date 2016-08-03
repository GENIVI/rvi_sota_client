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


#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub url:    Url,
    pub body:   Option<Vec<u8>>
}

pub type Response = Result<Vec<u8>, Error>;
