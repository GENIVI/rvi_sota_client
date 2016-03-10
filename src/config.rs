use hyper::Url;
use rustc_serialize::Decodable;
use std::fs::File;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::io;
use toml;

#[derive(Default, PartialEq, Eq, Debug)]
pub struct Config {
    pub auth: AuthConfig,
    pub ota:  OtaConfig,
    pub test: TestConfig,
}

#[derive(RustcDecodable, PartialEq, Eq, Debug, Clone)]
pub struct AuthConfig {
    pub server: Url,
    pub client_id: String,
    pub secret: String
}

#[derive(RustcDecodable, PartialEq, Eq, Debug, Clone)]
pub struct OtaConfig {
    pub server: Url,
    pub vin: String
}

#[derive(RustcDecodable, PartialEq, Eq, Debug)]
pub struct TestConfig {
    pub interpret: bool,
}

impl Default for AuthConfig {
    fn default() -> AuthConfig {
        AuthConfig {
            server: Url::parse("http://127.0.0.1:9000").unwrap(),
            client_id: "client-id".to_string(),
            secret: "secret".to_string(),
        }
    }
}

impl Default for OtaConfig {
    fn default() -> OtaConfig {
        OtaConfig {
            server: Url::parse("http://127.0.0.1:8080").unwrap(),
            vin: "V1234567890123456".to_string(),
        }
    }
}

impl Default for TestConfig {
    fn default() -> TestConfig {
        TestConfig {
            interpret: false,
        }
    }
}


pub fn parse_config(s: &str) -> Result<Config, io::Error> {

    fn parse_sect<T: Decodable>(tbl: &toml::Table, sect: &str) -> Result<T, io::Error> {
        tbl.get(sect)
            .and_then(|c| toml::decode::<T>(c.clone()) )
            .ok_or(Error::new(ErrorKind::Other,
                              "invalid section: ".to_string() + sect))
    }

    let tbl: toml::Table =
        try!(toml::Parser::new(&s)
             .parse()
             .ok_or(Error::new(ErrorKind::Other, "invalid toml")));

    let auth_cfg: AuthConfig = try!(parse_sect(&tbl, "auth"));
    let ota_cfg:  OtaConfig  = try!(parse_sect(&tbl, "ota"));
    let test_cfg: TestConfig = try!(parse_sect(&tbl, "test"));

    return Ok(Config {
        auth: auth_cfg,
        ota:  ota_cfg,
        test: test_cfg,
    })
}

pub fn load_config(path: &str) -> Config {

    fn helper(path: &str) -> Result<Config, io::Error> {
        let mut f = try!(File::open(path));
        let mut s = String::new();
        try!(f.read_to_string(&mut s));
        return parse_config(&s);
    }

    match helper(path) {
        Err(err) => {
            error!("Failed to load config: {}", err);
            return Config::default();
        },
        Ok(cfg) => return cfg
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    fn default_config_str() -> &'static str {
        r#"
        [auth]
        server = "http://127.0.0.1:9000"
        client_id = "client-id"
        secret = "secret"

        [ota]
        server = "http://127.0.0.1:8080"
        vin = "V1234567890123456"

        [test]
        interpret = false
        "#
    }

    #[test]
    fn parse_default_config() {
        assert_eq!(parse_config(default_config_str()).unwrap(),
                   Config::default());
    }

    fn bad_section_str() -> &'static str {
        r#"
        [uth]
        server = "http://127.0.0.1:9000"
        client_id = "client-id"
        secret = "secret"

        [ota]
        server = "http://127.0.0.1:8080"
        vin = "V1234567890123456"

        [test]
        interpret = false
        "#
    }

    #[test]
    fn bad_section() {
        assert!(parse_config(bad_section_str()).is_err())
    }

}
