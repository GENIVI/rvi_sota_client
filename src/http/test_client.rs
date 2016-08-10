use chan::Sender;
use std::cell::RefCell;

use datatype::Error;
use http::{Client, Request, Response};


pub struct TestClient {
    replies: RefCell<Vec<String>>
}

impl TestClient {
    pub fn new() -> TestClient {
        TestClient { replies: RefCell::new(Vec::new()) }
    }

    pub fn from(replies: Vec<String>) -> TestClient {
        TestClient { replies: RefCell::new(replies) }
    }
}

impl Client for TestClient {
    fn chan_request(&self, req: Request, resp_tx: Sender<Response>) {
        match self.replies.borrow_mut().pop() {
            Some(body) => resp_tx.send(Ok(body.as_bytes().to_vec())),
            None       => resp_tx.send(Err(Error::ClientError(req.url.to_string())))
        }
    }

    fn is_testing(&self) -> bool { true }
}