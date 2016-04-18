use rustc_serialize::json::Json;
use std::fs::File;
use std::io::{Write, Read};
use tempfile;

use datatype::{AccessToken, Error, Method, Url};


pub struct HttpRequest2<'a> {
    pub method: &'a Method,
    pub url:    &'a Url,
    pub token:  Option<&'a AccessToken>,
    pub body:   Option<&'a Json>
}

pub trait HttpClient2 {

    fn send_request_to(&self, request: &HttpRequest2, file: &File) -> Result<(), Error>;

    fn send_request(&self, request: &HttpRequest2) -> Result<String, Error> {

        let mut temp_file: File = try!(tempfile::tempfile());

        try!(Self::send_request_to(self, request, &temp_file));

        let mut buf = String::new();
        temp_file.read_to_string(&mut buf);

        Ok(buf)

    }

}
