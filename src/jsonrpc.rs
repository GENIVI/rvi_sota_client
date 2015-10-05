use time;

#[derive(RustcDecodable,RustcEncodable,Debug)]
pub struct Request<T> {
    pub jsonrpc: String,
    pub id: u64, // TODO: id can be any type
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
pub struct OkResponse<T> {
    pub jsonrpc: String,
    pub id: u64, // TODO: id can be any type
    pub result: Option<T>
}

impl<T> OkResponse<T> {
    pub fn new(id: u64, result: Option<T>) -> OkResponse<T> {
        OkResponse {
            jsonrpc: "2.0".to_string(),
            id: id,
            result: result
        }
    }
}

#[derive(RustcDecodable,RustcEncodable)]
pub struct ErrResponse {
    pub jsonrpc: String,
    pub id: u64,
    pub error: ErrorCode
}

impl ErrResponse {
    pub fn new(id: u64, error: ErrorCode) -> ErrResponse {
        ErrResponse {
            jsonrpc: "2.0".to_string(),
            id: id,
            error: error
        }
    }

    pub fn invalid_request(id: u64) -> ErrResponse {
        ErrResponse::new(id,
            ErrorCode {
                code: -32600,
                message: "Invalid Request".to_string()
            })
    }

    pub fn method_not_found(id: u64) -> ErrResponse {
        ErrResponse::new(id,
            ErrorCode {
                code: -32601,
                message: "Method not found".to_string()
            })
    }

    pub fn parse_error() -> ErrResponse {
        ErrResponse::new(0,
            ErrorCode {
                code: -32700,
                message: "Parse error".to_string()
            })
    }

    pub fn invalid_params(id: u64) -> ErrResponse {
        ErrResponse::new(
            id,
            ErrorCode {
                code: -32602,
                message: "Invalid params".to_string()
            })
    }
}

#[derive(RustcDecodable,RustcEncodable)]
pub struct ErrorCode {
    pub code: i32,
    pub message: String
}
