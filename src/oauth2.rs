use rustc_serialize::json;

use datatype::{AccessToken, Error, Url};
use http::{Client, Response};


/// Authenticate with the specified OAuth2 server to retrieve a new `AccessToken`.
pub fn authenticate(server: Url, client: &Client) -> Result<AccessToken, Error> {
    debug!("authenticating at {}", server);
    let resp_rx = client.post(server, Some(br#"grant_type=client_credentials"#.to_vec()));
    let resp    = resp_rx.recv().expect("no authenticate response received");
    let body    = match resp {
        Response::Success(data) => try!(String::from_utf8(data.body)),
        Response::Failed(data)  => return Err(Error::from(data)),
        Response::Error(err)    => return Err(err)
    };
    Ok(try!(json::decode(&body)))
}


#[cfg(test)]
mod tests {
    use super::*;
    use datatype::{AccessToken, Url};
    use http::TestClient;


    fn test_server() -> Url {
        "http://localhost:8000".parse().unwrap()
    }

    #[test]
    fn test_authenticate() {
        let token = r#"{
            "access_token": "token",
            "token_type": "type",
            "expires_in": 10,
            "scope": "scope1 scope2"
        }"#;
        let client = TestClient::from(vec![token.to_string()]);
        let expect = AccessToken {
            access_token: "token".to_string(),
            token_type:   "type".to_string(),
            expires_in:   10,
            scope:        "scope1 scope2".to_string()
        };
        assert_eq!(expect, authenticate(test_server(), &client).unwrap());
    }

    #[test]
    fn test_authenticate_bad_json() {
        let client = TestClient::from(vec![r#"{"apa": 1}"#.to_string()]);
        let expect = r#"Failed to decode JSON: MissingFieldError("access_token")"#;
        assert_eq!(expect, format!("{}", authenticate(test_server(), &client).unwrap_err()));
    }
}
