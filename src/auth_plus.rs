use hyper::header::{Authorization, Basic, ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use hyper;
use rustc_serialize::json;
use std::io::Read;

use config::AuthConfig;
use error::Error;


#[derive(Clone, RustcDecodable, Debug)]
pub struct AccessToken {
    pub access_token: String,
    token_type: String,
    expires_in: i32,
    scope: Option<String>
}

pub struct Client {
    hclient: hyper::Client,
    config: AuthConfig
}

impl Client {

    pub fn new(config: AuthConfig) -> Client {
        Client {
            hclient: hyper::Client::new(),
            config: config
        }
    }

    pub fn authenticate(&self) -> Result<AccessToken, Error> {

        self.hclient.post(self.config.server.join("/token").unwrap())
            .header(Authorization(Basic {
                username: self.config.client_id.clone(),
                password: Some(self.config.secret.clone()) }))
            .header(ContentType(Mime(
                TopLevel::Application,
                SubLevel::WwwFormUrlEncoded,
                vec![(Attr::Charset, Value::Utf8)])))
            .body("grant_type=client_credentials")
            .send()
            .map_err(|e| Error::AuthError(format!(
                "cannot send token request: {}", e)))
            .and_then(|mut resp| {
                let mut rbody = String::new();
                resp.read_to_string(&mut rbody)
                    .map_err(|e| Error::AuthError(format!(
                        "cannot read token response: {}", e)))
                    .and_then(|_| json::decode::<AccessToken>(&rbody)
                              .map_err(|e| Error::AuthError(format!(
                                  "cannot parse token response: {}. Got: {}", e, &rbody))))
            })
    }

}
