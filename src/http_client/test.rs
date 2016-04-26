use datatype::Error;
use http_client::{HttpClient, HttpRequest};


pub struct TestHttpClient<'a> {
    replies: Vec<&'a str>,
}

impl<'a> TestHttpClient<'a> {

    pub fn new() -> TestHttpClient<'a> {
        TestHttpClient { replies: Vec::new() }
    }

    pub fn from(replies: Vec<&'a str>) -> TestHttpClient<'a> {
        TestHttpClient { replies: replies }
    }

}

impl<'a> HttpClient for TestHttpClient<'a> {

    fn send_request(&self, req: &HttpRequest) -> Result<String, Error> {

        // XXX: this does't work... needs &mut self...
        let mut replies = self.replies.clone();

        replies.pop()
            .ok_or(Error::ClientError(req.to_string()))
            .map(|s| s.to_string())

    }

}
