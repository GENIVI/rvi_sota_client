use std::io::Write;

use datatype::Error;
use http_client::{HttpClient, HttpRequest};


pub struct MockHttpClient;

impl HttpClient for MockHttpClient {
    fn new() -> MockHttpClient {
        MockHttpClient
    }

    fn send_request(&self, _: &HttpRequest) -> Result<String, Error> {
        return Ok("[\"pkgid\"]".to_string())
    }

    fn send_request_to<W: Write>(&self, _: &HttpRequest, _: W) -> Result<(), Error> {
        return Ok(())
    }
}

#[cfg(test)]
mod tests {
    use hyper;

    mock_connector!(MockRedirectPolicy {
        "http://127.0.0.1" =>       "HTTP/1.1 301 Redirect\r\n\
                                     Location: http://127.0.0.2\r\n\
                                     Server: mock1\r\n\
                                     \r\n\
                                    "
        "http://127.0.0.2" =>       "HTTP/1.1 302 Found\r\n\
                                     Location: https://127.0.0.3\r\n\
                                     Server: mock2\r\n\
                                     \r\n\
                                    "
        "https://127.0.0.3" =>      "HTTP/1.1 200 OK\r\n\
                                     Server: mock3\r\n\
                                     \r\n\
                                    "
    });

    #[test]
    fn test_redirect_followall() {
        let mut client = hyper::Client::with_connector(MockRedirectPolicy::default());
        client.set_redirect_policy(hyper::client::RedirectPolicy::FollowAll);

        let res = client.get("http://127.0.0.1").send().unwrap();
        assert_eq!(res.headers.get(), Some(&hyper::header::Server("mock3".to_owned())));
    }
}
