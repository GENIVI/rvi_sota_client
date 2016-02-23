use config::{AuthConfig, OtaConfig};

use std::io::Read;
use std::result::Result;

use hyper::header::{Authorization, Basic, Bearer, ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use hyper;

use rustc_serialize::json;

#[derive(Clone, RustcDecodable)]
struct AccessToken {
    access_token: String,
    token_type: String,
    expires_in: i32,
    scope: Option<String>
}

pub struct OtaClient {
    hclient: hyper::Client,
    auth_cfg: AuthConfig,
    ota_cfg: OtaConfig
}

impl OtaClient {
    pub fn new((a, o): (AuthConfig, OtaConfig)) -> OtaClient {
        OtaClient {
            hclient: hyper::Client::new(),
            auth_cfg: a,
            ota_cfg: o
        }
    }

    pub fn check_for_update(&self) {
        let _ = self.get_token()
            .and_then(|tk| {
                self.hclient.get(self.ota_cfg.server.clone())
                    .header(Authorization(Bearer { token: tk.access_token }))
                    .send()
                    .map_err(|e| error!("Cannot send check_for_update request: {}", e)) })
            .and_then(|mut resp| {
                let mut rbody = String::new();
                resp.read_to_string(&mut rbody)
                    .map_err(|e| error!("Cannot read check_for_update response: {}", e))
                    .and_then(|_| {
                        Ok(info!("Check for update: {}", rbody)) }) });
    }

    fn get_token(&self) -> Result<AccessToken, ()> {
        self.hclient.post(self.auth_cfg.server.clone())
            .header(Authorization(Basic {
                username: self.auth_cfg.client_id.clone(),
                password: Some(self.auth_cfg.secret.clone()) }))
            .header(ContentType(Mime(
                        TopLevel::Application,
                        SubLevel::WwwFormUrlEncoded,
                        vec![(Attr::Charset, Value::Utf8)])))
            .body("grant_type=client_credentials")
            .send()
            .map_err(|e| error!("Cannot send token request: {}", e))
            .and_then(|mut resp| {
                let mut rbody = String::new();
                resp.read_to_string(&mut rbody)
                    .map_err(|e| error!("Cannot read token response: {}", e))
                    .and_then(|_| {
                        json::decode::<AccessToken>(&rbody)
                            .map_err(|e| error!("Cannot parse token response: {}", e)) }) })
    }
}
