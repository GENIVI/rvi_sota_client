use hyper::header::{Authorization, Basic, ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use rustc_serialize::json;

use config::AuthConfig;
use error::Error;
use http_client::{HttpClient, HttpRequest};
use access_token::AccessToken;


pub fn authenticate<C: HttpClient>(config: AuthConfig) -> Result<AccessToken, Error> {

    let http_client = C::new();

    let req = HttpRequest::post(config.server.join("/token").unwrap())
        .with_body("grant_type=client_credentials")
        .with_header(Authorization(Basic {
            username: config.client_id.clone(),
            password: Some(config.secret.clone())
        }))
        .with_header(ContentType(Mime(
            TopLevel::Application,
            SubLevel::WwwFormUrlEncoded,
            vec![(Attr::Charset, Value::Utf8)])));

    http_client.send_request(&req)
        .map_err(|e| Error::AuthError(format!("Can't get AuthPlus token: {}", e)))
        .and_then(|body| {
            return json::decode(&body)
                .map_err(|e| Error::ParseError(format!("Cannot parse response: {}. Got: {}", e, &body)))
        })

}

#[cfg(test)]
mod tests {

    use super::*;
    use access_token::AccessToken;
    use http_client::{HttpRequest, HttpClient};
    use error::Error;
    use config::AuthConfig;

    use hyper::header::{Authorization, Basic, ContentType};
    use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};

    struct MockClient {
        username: String,
        secret: String
    }

    impl MockClient {
        fn new(username: String, secret: String) -> MockClient {
            MockClient { username: username, secret: secret }
        }

        fn assert_authenticated(&self, req: &HttpRequest) {
            assert_eq!(req.body, Some("grant_type=client_credentials"));
            assert_eq!(Some(&Authorization(Basic { username: self.username.clone(), password: Some(self.secret.clone()) })),
                       req.headers.get::<Authorization<Basic>>())
        }

        fn assert_form_encoded(&self, req: &HttpRequest) {
            assert_eq!(Some(&ContentType(Mime(TopLevel::Application, SubLevel::WwwFormUrlEncoded,
                                              vec![(Attr::Charset, Value::Utf8)]))),
                       req.headers.get::<ContentType>())
        }
    }

    #[test]
    fn test_authenticate() {

        impl HttpClient for MockClient {

            fn new() -> MockClient {
                MockClient::new("".to_string(), "".to_string())
            }

            fn send_request(&self, req: &HttpRequest) -> Result<String, Error> {
                self.assert_authenticated(req);
                self.assert_form_encoded(req);
                return Ok(r#"{"access_token": "token",
                              "token_type": "type",
                              "expires_in": 10,
                              "scope": ["scope"]}"#.to_string())
            }
        }

        assert_eq!(authenticate::<MockClient>(AuthConfig::default()).unwrap(),
                   AccessToken {
                       access_token: "token".to_string(),
                       token_type: "type".to_string(),
                       expires_in: 10,
                       scope: vec!["scope".to_string()]
                   })
    }
}
