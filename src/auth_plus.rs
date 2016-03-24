use hyper::header::{Authorization, Basic, ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use rustc_serialize::json;

use datatype::AccessToken;
use datatype::AuthConfig;
use datatype::Error;
use http_client::{HttpClient, HttpRequest};


pub fn authenticate<C: HttpClient>(config: &AuthConfig) -> Result<AccessToken, Error> {

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

    let body = try!(C::new().send_request(&req)
                    .map_err(|e| Error::AuthError(format!("didn't receive access token: {}", e))));

    return Ok(try!(json::decode(&body)))

}

#[cfg(test)]
mod tests {

    use std::io::Write;

    use super::*;
    use datatype::AccessToken;
    use datatype::AuthConfig;
    use datatype::Error;
    use http_client::BadHttpClient;
    use http_client::{HttpRequest, HttpClient};


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
        assert_eq!(authenticate::<MockClient>(&AuthConfig::default()).unwrap(),
                   AccessToken {
                       access_token: "token".to_string(),
                       token_type: "type".to_string(),
                       expires_in: 10,
                       scope: vec!["scope".to_string()]
                   })
    }

    #[test]
    fn test_authenticate_bad_client() {
        assert_eq!(format!("{}", authenticate::<BadHttpClient>(&AuthConfig::default()).unwrap_err()),
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

        assert_eq!(format!("{}", authenticate::<BadJsonClient>(&AuthConfig::default()).unwrap_err()),
                   r#"Failed to decode JSON: MissingFieldError("access_token")"#)
    }

}
