use time;

#[derive(RustcDecodable,RustcEncodable,Debug)]
pub struct Request<T> {
    pub jsonrpc: String,
    pub id: u64, /// TODO: id can be any type
    pub method: String,
    pub params: T
}

impl<T> Request<T> {
    pub fn new(s: &str, p: T) -> Request<T> {
        Request::<T> {
            jsonrpc: "2.0".to_string(),
            id: time::precise_time_ns(),
            method: s.to_string(),
            params: p
        }
    }
}

#[derive(RustcDecodable,RustcEncodable)]
pub struct Response<T> {
    pub jsonrpc: String,
    pub id: String,
    pub result: Option<T>
}

