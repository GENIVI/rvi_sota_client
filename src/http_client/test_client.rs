use http_client::{HttpClient, HttpRequest, HttpResponse};
use std::cell::RefCell;
use std::sync::mpsc::Sender;

use datatype::Error;


pub struct TestHttpClient {
    replies: RefCell<Vec<Vec<u8>>>
}

impl TestHttpClient {
    pub fn new() -> TestHttpClient {
        TestHttpClient { replies: RefCell::new(Vec::new()) }
    }

    pub fn from(replies: Vec<Vec<u8>>) -> TestHttpClient {
        TestHttpClient { replies: RefCell::new(replies) }
    }
}

impl HttpClient for TestHttpClient {
    fn chan_request(&self, req: HttpRequest, resp_tx: Sender<HttpResponse>) {
        match self.replies.borrow_mut().pop() {
            Some(body) => { let _ = resp_tx.send(Ok(body)); }
            None       => { let _ = resp_tx.send(Err(Error::ClientError(req.url.to_string()))); }
        }
    }
}
