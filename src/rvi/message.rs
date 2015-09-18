use std::vec::Vec;
use time;

#[derive(RustcDecodable, RustcEncodable)]
pub struct Message<T> {
    pub service_name: String,
    pub parameters: Vec<T>
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct RVIMessage<T> {
    pub service_name: String,
    pub timeout: i64,
    pub parameters: Vec<T>
}

impl<T> RVIMessage<T> {
    pub fn new(service: &str,
               parameters: Vec<T>,
               tdelta: i64) -> RVIMessage<T> {
        let timeout = time::Duration::seconds(tdelta);
        RVIMessage {
            timeout: (time::get_time() + timeout).sec,
            service_name: service.to_string(),
            parameters: parameters
        }
    }
}

#[derive(RustcEncodable)]
pub struct RegisterServiceRequest {
    pub network_address: String,
    pub service: String
}

#[derive(RustcDecodable)]
pub struct RegisterServiceResponse {
    pub status: i32,
    pub service: String
}
