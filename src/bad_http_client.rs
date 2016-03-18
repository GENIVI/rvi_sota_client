use std::io::Write;

use error::Error;
use http_client::{HttpClient, HttpRequest};


pub struct BadHttpClient;

impl HttpClient for BadHttpClient {

    fn new() -> BadHttpClient {
        BadHttpClient
    }

    fn send_request(&self, _: &HttpRequest) -> Result<String, Error> {
        Err(Error::ClientError("bad client.".to_string()))
    }

    fn send_request_to<W: Write>(&self, _: &HttpRequest, _: W) -> Result<(), Error> {
        Err(Error::ClientError("bad client.".to_string()))
    }

}
