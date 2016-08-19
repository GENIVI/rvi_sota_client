use chan::Sender;
use std::cell::RefCell;

use datatype::Error;
use http::{Client, Request, Response};


/// The `TestClient` will return HTTP responses from an existing list of strings.
pub struct TestClient {
    responses: RefCell<Vec<String>>
}

impl Default for TestClient {
    fn default() -> Self {
        TestClient { responses: RefCell::new(Vec::new()) }
    }
}

impl TestClient {
    /// Create a new `TestClient` that will return these responses.
    pub fn from(responses: Vec<String>) -> TestClient {
        TestClient { responses: RefCell::new(responses) }
    }
}

impl Client for TestClient {
    fn chan_request(&self, req: Request, resp_tx: Sender<Response>) {
        match self.responses.borrow_mut().pop() {
            Some(body) => resp_tx.send(Ok(body.as_bytes().to_vec())),
            None       => resp_tx.send(Err(Error::Client(req.url.to_string())))
        }
    }

    fn is_testing(&self) -> bool { true }
}
