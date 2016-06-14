use std::sync::mpsc::{Sender, Receiver, channel};

use datatype::{Error, Method, Url};


pub trait HttpClient {
    fn send_request(&self, req: HttpRequest) -> Receiver<HttpResponse> {
        let (resp_tx, resp_rx): (Sender<HttpResponse>, Receiver<HttpResponse>) = channel();
        self.chan_request(req, resp_tx);
        resp_rx
    }

    fn chan_request(&self, req: HttpRequest, resp_tx: Sender<HttpResponse>);

    fn is_testing(&self) -> bool { false }
}

#[derive(Debug)]
pub struct HttpRequest {
    pub method: Method,
    pub url:    Url,
    pub body:   Option<Vec<u8>>
}

pub type HttpResponse = Result<Vec<u8>, Error>;
