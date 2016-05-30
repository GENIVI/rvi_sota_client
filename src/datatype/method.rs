use std::borrow::Cow;
use hyper::method;


#[derive(Clone, Debug)]
pub enum Method {
    Get,
    Post,
    Put,
}

impl ToString for Method {
    fn to_string(&self) -> String {
        match *self {
            Method::Get  => "GET".to_string(),
            Method::Post => "POST".to_string(),
            Method::Put  => "PUT".to_string(),
        }
    }
}

impl Into<method::Method> for Method {
    fn into(self) -> method::Method {
        match self {
            Method::Get  => method::Method::Get,
            Method::Post => method::Method::Post,
            Method::Put  => method::Method::Put,
        }
    }
}

impl<'a> Into<Cow<'a, Method>> for Method {
    fn into(self) -> Cow<'a, Method> {
        Cow::Owned(self)
    }
}
