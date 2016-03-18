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
        .map_err(|e| Error::AuthError(format!("didn't receive access token: {}", e)))
        .and_then(|body| {
            return json::decode(&body)
                .map_err(|e| Error::ParseError(format!(
                    "couldn't parse access token: {}. Got: {}.", e, &body)))
        })

}

#[cfg(test)]
mod tests {

    use super::*;

    use access_token::AccessToken;
    use bad_http_client::BadHttpClient;
    use config::AuthConfig;
    use error::Error;
    use http_client::{HttpRequest, HttpClient};

    use std::io::Write;

    struct MockClient;

    impl HttpClient for MockClient {

        fn new() -> MockClient {
            MockClient
        }

        fn send_request(&self, _: &HttpRequest) -> Result<String, Error> {
            Ok(r#"{"access_token": "token",
                   "token_type": "type",
                   "expires_in": 10,
                   "scope": ["scope"]}"#.to_string())
        }

        fn send_request_to<W: Write>(&self, _: &HttpRequest, _: W) -> Result<(), Error> {
            Ok(())
        }
    }

    #[test]
    fn test_authenticate() {
        assert_eq!(authenticate::<MockClient>(AuthConfig::default()).unwrap(),
                   AccessToken {
                       access_token: "token".to_string(),
                       token_type: "type".to_string(),
                       expires_in: 10,
                       scope: vec!["scope".to_string()]
                   })
    }

    #[test]
    fn test_authenticate_bad_client() {
        assert_eq!(format!("{}", authenticate::<BadHttpClient>(AuthConfig::default()).unwrap_err()),
                   "Authentication error, didn't receive access token: bad client.")
    }

    #[test]
    fn test_authenticate_bad_json_client() {

        struct BadJsonClient;

        impl HttpClient for BadJsonClient {

            fn new() -> BadJsonClient {
                BadJsonClient
            }

            fn send_request(&self, _: &HttpRequest) -> Result<String, Error> {
                Ok(r#"{"apa": 1}"#.to_string())
            }

            fn send_request_to<W: Write>(&self, _: &HttpRequest, _: W) -> Result<(), Error> {
                Ok(())
            }
        }

        assert_eq!(format!("{}", authenticate::<BadJsonClient>(AuthConfig::default()).unwrap_err()),
                   r#"couldn't parse access token: MissingFieldError("access_token"). Got: {"apa": 1}."#)
    }

}
