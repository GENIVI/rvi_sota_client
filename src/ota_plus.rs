use hyper::header::{Authorization, Bearer, ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};

use http_client::{HttpClient, HttpRequest};
use rustc_serialize::json;
use std::result::Result;

use access_token::AccessToken;
use config::OtaConfig;
use error::Error;
use package::Package;

pub struct Client<C: HttpClient> {
    http_client: C,
    access_token: String,
    config: OtaConfig
}

impl<C: HttpClient> Client<C> {

    pub fn new(client: C, token: AccessToken, config: OtaConfig) -> Client<C> {
        Client {
            http_client: client,
            access_token: token.access_token,
            config: config
        }
    }

    #[allow(dead_code)]
    pub fn check_for_update(&self) -> Result<String, Error> {
        let req = HttpRequest::get(self.config.server.join("/updates").unwrap())
            .with_header(Authorization(Bearer { token: self.access_token.clone() }));
        self.http_client.send_request(&req)
    }

    pub fn post_packages(&self, pkgs: Vec<Package>) -> Result<(), Error> {
        json::encode(&pkgs)
            .map_err(|_| Error::ParseError(String::from("JSON encoding error")))
            .and_then(|json| {
                let req = HttpRequest::post(self.config.server.join("/packages").unwrap())
                    .with_header(Authorization(Bearer { token: self.access_token.clone() }))
                    .with_header(ContentType(Mime(
                        TopLevel::Application,
                        SubLevel::Json,
                        vec![(Attr::Charset, Value::Utf8)])))
                    .with_body(&json);

                self.http_client.send_request(&req).map(|_| ())
            })
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use http_client::{HttpRequest, HttpClient};
    use error::Error;
    use package::Package;
    use config::OtaConfig;
    use access_token::AccessToken;

    use hyper::header::{Authorization, Bearer};

    struct MockClient {
        access_token: String
    }

    impl MockClient {
        fn new(token: AccessToken) -> MockClient {
            MockClient { access_token: token.access_token }
        }

        fn assert_authenticated(&self, req: &HttpRequest) {
            assert_eq!(Some(&Authorization(Bearer { token: self.access_token.clone() })),
                       req.headers.get::<Authorization<Bearer>>())
        }
    }

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

    #[test]
    fn test_post_packages_sends_authentication() {

        impl HttpClient for MockClient {
            fn send_request(&self, req: &HttpRequest) -> Result<String, Error> {
                self.assert_authenticated(req);
                Ok::<String, Error>("ok".to_string())
            }
        }

        let mock = MockClient::new(test_token());
        let ota_plus = Client::new(mock, test_token(), OtaConfig::default());

        let _ = ota_plus.post_packages(vec![test_package()]);
    }
}
