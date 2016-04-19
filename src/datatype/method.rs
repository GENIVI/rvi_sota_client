use hyper::method;


#[derive(Clone)]
pub enum Method {
    Get,
    Post,
}

impl Into<method::Method> for Method {
    fn into(self) -> method::Method {
        match self {
            Method::Get  => method::Method::Get,
            Method::Post => method::Method::Post,
        }
    }
}
