use hyper::Url;
use rustc_serialize::Decodable;
use std::fs;
use std::fs::File;
use std::io::ErrorKind;
use std::io::prelude::*;
use std::path::Path;
use toml;

use datatype::Error;
use datatype::error::ConfigReason::{Parse, Io};
use datatype::error::ParseReason::{InvalidToml, InvalidSection};
use package_manager::PackageManager;


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
    pub secret: String,
}

impl AuthConfig {
    fn new(server: Url, creds: CredentialsFile) -> AuthConfig {
        AuthConfig { server: server,
                     client_id: creds.client_id,
                     secret: creds.secret }
    }
}

#[derive(RustcDecodable, PartialEq, Eq, Debug, Clone)]
struct AuthConfigSection {
    pub server: Url,
    pub client_id: String,
    pub secret: String,
    pub credentials_file: String,
}

#[derive(RustcEncodable, RustcDecodable, PartialEq, Eq, Debug, Clone)]
struct CredentialsFile {
    pub client_id: String,
    pub secret: String,
}

#[derive(RustcDecodable, PartialEq, Eq, Debug, Clone)]
pub struct OtaConfig {
    pub server: Url,
    pub vin: String,
    pub polling_interval: u64,
    pub packages_dir: String,
    pub package_manager: PackageManager,
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

impl Default for AuthConfigSection {
    fn default() -> AuthConfigSection {
        AuthConfigSection {
            server: Url::parse("http://127.0.0.1:9000").unwrap(),
            client_id: "client-id".to_string(),
            secret: "secret".to_string(),
            credentials_file: "/tmp/ats_credentials.toml".to_string(),
        }
    }
}

impl Default for OtaConfig {
    fn default() -> OtaConfig {
        OtaConfig {
            server: Url::parse("http://127.0.0.1:8080").unwrap(),
            vin: "V1234567890123456".to_string(),
            polling_interval: 10,
            packages_dir: "/tmp/".to_string(),
            package_manager: PackageManager::Dpkg,
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

fn parse_toml(s: &str) -> Result<toml::Table, Error> {
    let table: toml::Table = try!(toml::Parser::new(&s)
                                  .parse()
                                  .ok_or(Error::Config(Parse(InvalidToml))));
    Ok(table)
}

fn parse_toml_table<T: Decodable>(tbl: &toml::Table, sect: &str) -> Result<T, Error> {
    tbl.get(sect)
        .and_then(|c| toml::decode::<T>(c.clone()) )
        .ok_or(Error::Config(Parse(InvalidSection(sect.to_string()))))
}

fn bootstrap_credentials(auth_cfg_section: AuthConfigSection) -> Result<AuthConfig, Error> {

    fn persist_credentials_file(creds: &CredentialsFile, path: &Path) -> Result<(), Error> {
        let mut tbl = toml::Table::new();
        tbl.insert("auth".to_string(), toml::encode(&creds));
        try!(fs::create_dir_all(&path.parent().unwrap()));
        let mut f = try!(File::create(path));
        try!(f.write_all(&toml::encode_str(&tbl).into_bytes()));
        Ok(())
    }

    fn read_credentials_file(mut f: File) -> Result<CredentialsFile, Error> {
        let mut s = String::new();
        try!(f.read_to_string(&mut s)
             .map_err(|err| Error::Config(Io(err))));
        let toml_table = try!(parse_toml(&s));
        let creds: CredentialsFile = try!(parse_toml_table(&toml_table, "auth"));
        Ok(creds)
    }

    let creds_path = Path::new(&auth_cfg_section.credentials_file);

    match File::open(creds_path) {
        Err(ref e) if e.kind() == ErrorKind::NotFound => {
            let creds = CredentialsFile { client_id: auth_cfg_section.client_id,
                                          secret: auth_cfg_section.secret };
            try!(persist_credentials_file(&creds, &creds_path));
            Ok(AuthConfig::new(auth_cfg_section.server, creds))
        }
        Err(e)                                        => Err(Error::Config(Io(e))),
        Ok(f)                                         => {
            let creds = try!(read_credentials_file(f));
            Ok(AuthConfig::new(auth_cfg_section.server, creds))
        }
    }
}

pub fn parse_config(s: &str) -> Result<Config, Error> {
    let tbl = try!(parse_toml(&s));
    let auth_cfg_section: AuthConfigSection = try!(parse_toml_table(&tbl, "auth"));
    let auth_cfg: AuthConfig = try!(bootstrap_credentials(auth_cfg_section));
    let ota_cfg:  OtaConfig  = try!(parse_toml_table(&tbl, "ota"));
    let test_cfg: TestConfig = try!(parse_toml_table(&tbl, "test"));

    return Ok(Config {
        auth: auth_cfg,
        ota:  ota_cfg,
        test: test_cfg,
    })
}

pub fn load_config(path: &str) -> Result<Config, Error> {

    match File::open(path) {
        Err(ref e) if e.kind() == ErrorKind::NotFound => Ok(Config::default()),
        Err(e)                                        => Err(Error::Config(Io(e))),
        Ok(mut f)                                     => {
            let mut s = String::new();
            try!(f.read_to_string(&mut s)
                 .map_err(|err| Error::Config(Io(err))));
            return parse_config(&s);
        }
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    const DEFAULT_CONFIG_STRING: &'static str =
        r#"
        [auth]
        server = "http://127.0.0.1:9000"
        client_id = "client-id"
        secret = "secret"
        credentials_file = "/tmp/ats_credentials.toml"

        [ota]
        server = "http://127.0.0.1:8080"
        vin = "V1234567890123456"
        polling_interval = 10
        packages_dir = "/tmp/"
        package_manager = "dpkg"

        [test]
        looping = false
        "#;

    #[test]
    fn parse_default_config() {
        assert_eq!(parse_config(DEFAULT_CONFIG_STRING).unwrap(),
                   Config::default());
    }

    #[test]
    fn load_default_config() {
        assert_eq!(load_config("ota.toml").unwrap(),
                   Config::default());
    }

    #[test]
    fn bad_path_yields_default_config() {
        assert_eq!(load_config("").unwrap(),
                   Config::default())
    }



}
