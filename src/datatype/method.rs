use hyper::method;


pub enum Method {
    Get,
    Post,
}

impl<'a> Into<method::Method> for &'a Method {
    fn into(self) -> method::Method {
        match *self {
            Method::Get  => method::Method::Get,
            Method::Post => method::Method::Post,
        }
    }
}
