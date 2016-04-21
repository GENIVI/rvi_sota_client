use std::borrow::Cow;
use std::fs::File;
use std::io::{Write, Read};
use tempfile;

use datatype::{AccessToken, ClientId, ClientSecret, Error, Method, Url};


#[derive(Clone)]
pub enum Auth<'a> {
    Credentials(ClientId, ClientSecret),
    Token(&'a AccessToken),
}

impl<'a> Into<Cow<'a, Auth<'a>>> for Auth<'a> {
    fn into(self) -> Cow<'a, Auth<'a>> {
        Cow::Owned(self)
    }
}

pub struct HttpRequest2<'a> {
    pub method: Cow<'a, Method>,
    pub url:    Cow<'a, Url>,
    pub auth:   Cow<'a, Auth<'a>>,
    pub body:   Option<Cow<'a, str>>,
}

impl<'a> HttpRequest2<'a> {

    fn new<M, U, A, B>(meth: M,
                       url:  U,
                       auth: A,
                       body: Option<B>) -> HttpRequest2<'a>
        where
        M: Into<Cow<'a, Method>>,
        U: Into<Cow<'a, Url>>,
        A: Into<Cow<'a, Auth<'a>>>,
        B: Into<Cow<'a, str>>
    {
        HttpRequest2 {
            method: meth.into(),
            url:    url.into(),
            auth:   auth.into(),
            body:   body.map(|c| c.into()),
        }
    }

    pub fn get<U, A>(url: U, auth: A) -> HttpRequest2<'a>
        where
        U: Into<Cow<'a, Url>>,
        A: Into<Cow<'a, Auth<'a>>>,
    {
        HttpRequest2::new::<_, _, _, String>(Method::Get, url, auth, None)
    }

    pub fn post<U, A, B>(url: U, auth: A, body: Option<B>) -> HttpRequest2<'a>
        where
        U: Into<Cow<'a, Url>>,
        A: Into<Cow<'a, Auth<'a>>>,
        B: Into<Cow<'a, str>>
    {
        HttpRequest2::new(Method::Post, url, auth, body)
    }

}

pub trait HttpClient2 {

    fn send_request_to(&self, request: &HttpRequest2, file: &mut File) -> Result<(), Error> {

        let s = try!(Self::send_request(self, request));

        Ok(try!(file.write_all(&s.as_bytes())))

    }

    fn send_request(&self, request: &HttpRequest2) -> Result<String, Error> {

        let mut temp_file: File = try!(tempfile::tempfile());

        try!(Self::send_request_to(self, request, &mut temp_file));

        let mut buf = String::new();
        let _: usize = try!(temp_file.read_to_string(&mut buf));

        Ok(buf)

    }

}
