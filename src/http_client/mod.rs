pub use self::http_client::{Auth, HttpClient, HttpRequest};
pub use self::hyper::Hyper;

pub mod mock_http_client;
pub mod http_client;
pub mod hyper;
