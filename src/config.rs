use hyper::Url;
use rustc_serialize::Decodable;
use std::fs::File;
use std::io::prelude::*;
use std::io::ErrorKind;
use std::io;
use toml;

use error::Error;


#[derive(Default, PartialEq, Eq, Debug, Clone)]
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

#[derive(RustcDecodable, PartialEq, Eq, Debug, Clone)]
pub struct TestConfig {
    pub looping: bool,
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
            looping: false,
        }
    }
}


pub fn parse_config(s: &str) -> Result<Config, Error> {

    fn parse_sect<T: Decodable>(tbl: &toml::Table, sect: &str) -> Result<T, Error> {
        tbl.get(sect)
            .and_then(|c| toml::decode::<T>(c.clone()) )
            .ok_or(Error::ConfigParseError(format!("invalid section: {}", sect)))
    }

    let tbl: toml::Table =
        try!(toml::Parser::new(&s)
             .parse()
             .ok_or(Error::ConfigParseError("invalid toml".to_string())));

    let auth_cfg: AuthConfig = try!(parse_sect(&tbl, "auth"));
    let ota_cfg:  OtaConfig  = try!(parse_sect(&tbl, "ota"));
    let test_cfg: TestConfig = try!(parse_sect(&tbl, "test"));

    return Ok(Config {
        auth: auth_cfg,
        ota:  ota_cfg,
        test: test_cfg,
    })
}

pub fn load_config(path: &str) -> Result<Config, Error> {

    impl From<io::Error> for Error {
        fn from(err: io::Error) -> Error {
            Error::ConfigIOError(format!("{}", err))
        }
    }

    match File::open(path) {
        Err(ref e) if e.kind() == ErrorKind::NotFound => Ok(Config::default()),
        Err(e) => Err(From::from(e)),
        Ok(mut f) => {
            let mut s = String::new();
            try!(f.read_to_string(&mut s));
            return parse_config(&s);
        }
    }
}


#[cfg(test)]
mod tests {

    use super::*;
    use error::Error;

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
        looping = false
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
        looping = false
        "#
    }

    #[test]
    fn bad_section() {
        assert_eq!(parse_config(bad_section_str()),
                   Err(Error::ConfigParseError("invalid section: auth".to_string())))
    }

    #[test]
    fn bad_path_yields_default_config() {
        assert_eq!(load_config(""), Ok(Config::default()))
    }

    #[test]
    fn bad_path_dir() {
        assert_eq!(load_config("/"),
                   Err(Error::ConfigIOError(
                       "Is a directory (os error 21)".to_string())))
    }

    fn bad_toml_str() -> &'static str {
        r#"
        auth]
        "#
    }

    #[test]
    fn bad_toml() {
        assert_eq!(parse_config(bad_toml_str()),
                   Err(Error::ConfigParseError(
                       "invalid toml".to_string())))
    }

}
