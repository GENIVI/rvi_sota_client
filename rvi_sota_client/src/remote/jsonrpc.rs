//! RVI specific implementation of the jsonrpc protocol
use time;

/// Type to encode a generic jsonrpc call.
#[derive(RustcDecodable,RustcEncodable,Debug)]
pub struct Request<T> {
    /// The version of jsonrpc to use, has to be set to `"2.0"`.
    pub jsonrpc: String,
    /// The identifier of the request. Only unsigned numbers are accepted
    // TODO: id can be any type
    pub id: u64,
    /// The method to call on the receiving side.
    pub method: String,
    /// Arguments for the method.
    pub params: T
}

impl<T> Request<T> {
    /// Returns a new `Request`.
    ///
    /// # Arguments
    /// * `s`: The name of the method to call.
    /// * `p`: The arguments of said method.
    pub fn new(s: &str, p: T) -> Request<T> {
        Request::<T> {
            jsonrpc: "2.0".to_string(),
            id: time::precise_time_ns(),
            method: s.to_string(),
            params: p
        }
    }
}

/// Response to a jsonrpc call, indicating a successful method call.
#[derive(RustcDecodable,RustcEncodable)]
pub struct OkResponse<T> {
    /// The version of jsonrpc to use, has to be set to `"2.0"`.
    pub jsonrpc: String,
    /// The identifier of the jsonrpc call this response belongs to. Only unsigned numbers are
    /// accepted
    // TODO: id can be any type
    pub id: u64,
    /// The result of the method call, if any.
    pub result: Option<T>
}

impl<T> OkResponse<T> {
    /// Returns a new `OkResponse`
    ///
    /// # Arguments
    /// * `id`: The identifier of the jsonrpc call the returned response belongs to.
    /// * `result`: The result of the method call, if any.
    pub fn new(id: u64, result: Option<T>) -> OkResponse<T> {
        OkResponse {
            jsonrpc: "2.0".to_string(),
            id: id,
            result: result
        }
    }
}

/// Response to a jsonrpc call, indicating failure.
#[derive(RustcDecodable,RustcEncodable)]
pub struct ErrResponse {
    /// The version of jsonrpc to use, has to be set to `"2.0"`.
    pub jsonrpc: String,
    /// The identifier of the jsonrpc call this response belongs to. Only unsigned numbers are
    /// accepted
    // TODO: id can be any type
    pub id: u64,
    /// The error code and message.
    pub error: ErrorCode
}

impl ErrResponse {
    /// Returns a new `ErrResponse`
    ///
    /// # Arguments
    /// * `id`: The identifier of the jsonrpc call the returned response belongs to.
    /// * `error`: The error code and message. See [`ErrorCode`](./struct.ErrorCode.html).
    pub fn new(id: u64, error: ErrorCode) -> ErrResponse {
        ErrResponse {
            jsonrpc: "2.0".to_string(),
            id: id,
            error: error
        }
    }

    /// Returns a new `ErrResponse`, indicating a ["Invalid
    /// Request"](http://www.jsonrpc.org/specification#error_object) error.
    pub fn invalid_request(id: u64) -> ErrResponse {
        ErrResponse::new(id,
            ErrorCode {
                code: -32600,
                message: "Invalid Request".to_string()
            })
    }

    /// Returns a new `ErrResponse`, indicating a ["Method not
    /// found"](http://www.jsonrpc.org/specification#error_object) error.
    pub fn method_not_found(id: u64) -> ErrResponse {
        ErrResponse::new(id,
            ErrorCode {
                code: -32601,
                message: "Method not found".to_string()
            })
    }

    /// Returns a new `ErrResponse`, indicating a ["Parse
    /// error"](http://www.jsonrpc.org/specification#error_object) error.
    pub fn parse_error() -> ErrResponse {
        ErrResponse::new(0,
            ErrorCode {
                code: -32700,
                message: "Parse error".to_string()
            })
    }

    /// Returns a new `ErrResponse`, indicating a ["Invalid
    /// params"](http://www.jsonrpc.org/specification#error_object) error.
    pub fn invalid_params(id: u64) -> ErrResponse {
        ErrResponse::new(
            id,
            ErrorCode {
                code: -32602,
                message: "Invalid params".to_string()
            })
    }

    /// Returns a new `ErrResponse`, indicating a unspecified error.
    pub fn unspecified(id: u64) -> ErrResponse {
        ErrResponse::new(
            id,
            ErrorCode {
                code: -32100,
                message: "Couldn't handle request".to_string()
            })
    }
}

/// Type to encode a jsonrpc error.
#[derive(RustcDecodable,RustcEncodable)]
pub struct ErrorCode {
    /// The error code as [specified by
    /// jsonrpc](http://www.jsonrpc.org/specification#error_object).
    pub code: i32,
    /// The error message as [specified by
    /// jsonrpc](http://www.jsonrpc.org/specification#error_object).
    pub message: String
}
