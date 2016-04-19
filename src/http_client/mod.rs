pub use self::mock_http_client::MockHttpClient;
pub use self::bad_http_client::BadHttpClient;
pub use self::interface::{HttpClient, HttpRequest};

pub mod bad_http_client;
pub mod mock_http_client;
pub mod interface;
