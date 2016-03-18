//! Wrappers for messages exchanged with RVI.

use std::vec::Vec;
use time;

/// A generic incoming message.
#[derive(RustcDecodable, RustcEncodable)]
pub struct Message<T> {
    /// The service that got called.
    pub service_name: String,
    /// The paramaters to the service call.
    pub parameters: Vec<T>
}

/// A generic outgoing message.
#[derive(RustcDecodable, RustcEncodable)]
pub struct RVIMessage<T> {
    /// The service name to call.
    pub service_name: String,
    /// A timestamp when this message should expire. In UTC UNIX epoch.
    pub timeout: i64,
    /// The parameters to the service call.
    pub parameters: Vec<T>
}

impl<T> RVIMessage<T> {
    /// Create a new outgoing RVI message.
    ///
    /// # Arguments
    /// * `service`: The service name to call.
    /// * `parameters`: The parameters to the service call.
    /// * `tdelta`: Amount of seconds before the message will expire.
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

/// Encodes a registration request.
#[derive(RustcEncodable)]
pub struct RegisterServiceRequest {
    /// The network address where RVI can be reached.
    pub network_address: String,
    /// The service (short name) to register.
    pub service: String
}

/// Encodes a registration response.
#[derive(RustcDecodable)]
pub struct RegisterServiceResponse {
    /// Status number indicating success or failure. See the RVI documentation for details.
    pub status: i32,
    /// The full service URL, that RVI assigned.
    pub service: String
}
