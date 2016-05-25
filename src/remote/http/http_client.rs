use std::borrow::Cow;

use super::datatype::{AccessToken, ClientId, ClientSecret, Error, Method, Url};

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

pub struct HttpRequest<'a> {
    pub method: Cow<'a, Method>,
    pub url:    Cow<'a, Url>,
    pub auth:   Option<Cow<'a, Auth<'a>>>,
    pub body:   Option<Cow<'a, str>>,
}

impl<'a> HttpRequest<'a> {

    fn new<M, U, A, B>(meth: M,
                       url:  U,
                       auth: Option<A>,
                       body: Option<B>) -> HttpRequest<'a>
        where
        M: Into<Cow<'a, Method>>,
        U: Into<Cow<'a, Url>>,
        A: Into<Cow<'a, Auth<'a>>>,
        B: Into<Cow<'a, str>>
    {
        HttpRequest {
            method: meth.into(),
            url:    url.into(),
            auth:   auth.map(|a| a.into()),
            body:   body.map(|b| b.into()),
        }
    }

    pub fn get<U, A>(url: U, auth: Option<A>) -> HttpRequest<'a>
        where
        U: Into<Cow<'a, Url>>,
        A: Into<Cow<'a, Auth<'a>>>,
    {
        HttpRequest::new::<Method, U, A, String>(Method::Get, url, auth, None)
    }

    pub fn post<U, A, B>(url: U, auth: Option<A>, body: Option<B>) -> HttpRequest<'a>
        where
        U: Into<Cow<'a, Url>>,
        A: Into<Cow<'a, Auth<'a>>>,
        B: Into<Cow<'a, str>>
    {
        HttpRequest::new(Method::Post, url, auth, body)
    }

    pub fn put<U, A, B>(url: U, auth: Option<A>, body: Option<B>) -> HttpRequest<'a>
        where
        U: Into<Cow<'a, Url>>,
        A: Into<Cow<'a, Auth<'a>>>,
        B: Into<Cow<'a, str>>
    {
        HttpRequest::new(Method::Put, url, auth, body)
    }
}

impl<'a> ToString for HttpRequest<'a> {
    fn to_string(&self) -> String {
        format!("{} {}", self.method.to_string(), self.url.to_string())
    }
}

#[derive(RustcEncodable, RustcDecodable)]
pub enum HttpStatus {
    Ok,
}

impl ToString for HttpStatus {
    fn to_string(&self) -> String {
        match *self {
            HttpStatus::Ok => "200".to_string()
        }
    }
}

#[derive(RustcEncodable, RustcDecodable)]
pub struct HttpResponse {
    pub status: HttpStatus,
    pub body:   Vec<u8>,
}

pub trait HttpClient: Send + Sync {

    fn send_request(&mut self, req: &HttpRequest) -> Result<HttpResponse, Error>;

}
