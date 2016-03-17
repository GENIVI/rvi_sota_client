use hyper::header::{Authorization, Bearer, ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use rustc_serialize::json;
use std::result::Result;

use access_token::AccessToken;
use config::OtaConfig;
use error::Error;
use http_client::{HttpClient, HttpRequest};
use package::Package;


#[allow(dead_code)]
pub fn check_for_update<C: HttpClient>(token: AccessToken,
                                       config: OtaConfig) -> Result<String, Error> {

    let http_client = C::new();

    let req = HttpRequest::get(config.server.join("/updates").unwrap())
        .with_header(Authorization(Bearer { token: token.access_token }));

    http_client.send_request(&req)

}

pub fn post_packages<C: HttpClient>(token: AccessToken,
                                    config: OtaConfig,
                                    pkgs: Vec<Package>) -> Result<(), Error> {

    let http_client = C::new();

    json::encode(&pkgs)
        .map_err(|_| Error::ParseError(String::from("JSON encoding error")))
        .and_then(|json| {
            let req = HttpRequest::post(config.server.join("/packages").unwrap())
                .with_header(Authorization(Bearer { token: token.access_token.clone() }))
                .with_header(ContentType(Mime(
                    TopLevel::Application,
                    SubLevel::Json,
                    vec![(Attr::Charset, Value::Utf8)])))
                .with_body(&json);

            http_client.send_request(&req).map(|_| ())
        })
}

#[cfg(test)]
mod tests {

    use super::*;
    use http_client::{HttpRequest, HttpClient};
    use error::Error;
    use package::Package;
    use config::OtaConfig;
    use access_token::AccessToken;


    fn test_token() -> AccessToken {
        AccessToken {
            access_token: "token".to_string(),
            token_type: "bar".to_string(),
            expires_in: 20,
            scope: vec![]
        }
    }

    fn test_package() -> Package {
        Package {
            name: "hey".to_string(),
            version: "1.2.3".to_string()
        }
    }

    struct MockClient {}

    impl HttpClient for MockClient {

        fn new() -> MockClient {
            MockClient {}
        }

        fn send_request(&self, _: &HttpRequest) -> Result<String, Error> {
            return Ok("ok".to_string())
        }

    }

    #[test]
    fn test_post_packages_sends_authentication() {
        assert_eq!(
            post_packages::<MockClient>(test_token(), OtaConfig::default(), vec![test_package()])
                .unwrap(), ())
    }
}
