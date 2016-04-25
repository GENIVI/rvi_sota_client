use rustc_serialize::json;

use datatype::{AccessToken, AuthConfig, ClientId, ClientSecret, Error};
use http_client::{Auth, HttpClient, HttpRequest};


pub fn authenticate(config: &AuthConfig, client: &HttpClient) -> Result<AccessToken, Error> {

    let req = HttpRequest::post::<_, _, String>(
        config.server.join("/token").unwrap(),
        Some(Auth::Credentials(
            ClientId     { get: config.client_id.clone() },
            ClientSecret { get: config.secret.clone() })),
        None,
    );

    let body = try!(client.send_request(&req));

    Ok(try!(json::decode(&body)))

}

/*

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
*/
