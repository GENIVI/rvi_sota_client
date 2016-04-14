use hyper::method;


pub enum Method {
    Get,
    Post,
}

impl Method {
    pub fn to_hyper(&self) -> method::Method {
        match *self {
            Method::Get  => method::Method::Get,
            Method::Post => method::Method::Post,
        }
    }
}
