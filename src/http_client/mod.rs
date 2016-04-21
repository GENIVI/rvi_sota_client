pub use self::mock_http_client::MockHttpClient;
pub use self::bad_http_client::BadHttpClient;
pub use self::interface::{HttpClient, HttpRequest};
pub use self::http_client::{Auth, HttpClient2, HttpRequest2};

pub mod bad_http_client;
pub mod mock_http_client;
pub mod http_client;
pub mod hyper;
pub mod interface;
