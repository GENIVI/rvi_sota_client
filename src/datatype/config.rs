use rustc_serialize::Decodable;
use std::fs;
use std::fs::File;
use std::io::ErrorKind;
use std::io::prelude::*;
use std::path::Path;
use toml;

use datatype::{Error, Url};
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
    pub vin: String,
}

impl AuthConfig {
    fn new(server: Url, creds: CredentialsFile) -> AuthConfig {
        AuthConfig {
            server: server,
            client_id: creds.client_id,
            secret: creds.secret,
            vin: creds.vin
        }
    }
}

#[derive(RustcDecodable, PartialEq, Eq, Debug, Clone)]
struct AuthConfigSection {
    pub server: Url,
    pub client_id: String,
    pub secret: String,
    pub credentials_file: String,
    pub vin: String,
}

#[derive(RustcEncodable, RustcDecodable, PartialEq, Eq, Debug, Clone)]
struct CredentialsFile {
    pub client_id: String,
    pub secret: String,
    pub vin: String,
}

#[derive(RustcDecodable, PartialEq, Eq, Debug, Clone)]
pub struct OtaConfig {
    pub server: Url,
    pub polling_interval: u64,
    pub packages_dir: String,
    pub package_manager: PackageManager,
}

#[derive(RustcDecodable, PartialEq, Eq, Debug, Clone)]
pub struct ClientConfiguration {
    /// Directory where chunks and packages will be stored.
    pub storage_dir: String,
    /// The full URL where RVI can be reached.
    pub rvi_url: Url,
    /// The `host:port` combination where the client should bind and listen for incoming RVI calls.
    pub edge_url: Url,
    /// How long to wait for further server messages before the `Transfer` will be dropped.
    pub timeout: Option<i64>,
    /// Index of the RVI service URL, that holds the VIN for this device.
    pub vin_match: i32
}

#[derive(RustcDecodable, PartialEq, Eq, Debug, Clone)]
pub struct DBusConfiguration {
    /// The DBus name that sota_client registers.
    pub name: String,
    /// The DBus path that sota_client registers.
    pub path: String,
    /// The interface name that sota_client provides.
    pub interface: String,
    /// The name and interface, where the software loading manager can be reached.
    pub software_manager: String,
    /// The name and interface, where the software loading manager can be reached.
    pub software_manager_path: String,
    /// Time to wait for installation of a package before it is considered a failure. In seconds.
    pub timeout: i32 // dbus-rs expects a signed int
}

#[derive(RustcDecodable, PartialEq, Eq, Debug, Clone)]
pub struct TestConfig {
    pub http: bool,
    pub repl: bool,
    pub websocket: bool,
}

impl Default for AuthConfig {
    fn default() -> AuthConfig {
        AuthConfig {
            server: Url::parse("http://127.0.0.1:9000").unwrap(),
            client_id: "client-id".to_string(),
            secret: "secret".to_string(),
            vin: "V1234567890123456".to_string(),
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
            vin: "V1234567890123456".to_string(),
        }
    }
}

impl Default for OtaConfig {
    fn default() -> OtaConfig {
        OtaConfig {
            server: Url::parse("http://127.0.0.1:8080").unwrap(),
            polling_interval: 10,
            packages_dir: "/tmp/".to_string(),
            package_manager: PackageManager::Dpkg,
        }
    }
}

impl Default for DBusConfiguration {
    fn default() -> DBusConfiguration {
        DBusConfiguration {
            name: "org.genivi.SotaClient".to_string(),
            path: "/org/genivi/SotaClient".to_string(),
            interface: "org.genivi.SotaClient".to_string(),
            software_manager: "org.genivi.SoftwareLoadingManager".to_string(),
            software_manager_path: "/org/genivi/SoftwareLoadingManager".to_string(),
            timeout: 60
        }
    }
}

impl Default for ClientConfiguration {
    fn default() -> ClientConfiguration {
        ClientConfiguration {
            storage_dir: "/var/sota".to_string(),
            rvi_url: Url::parse("http://127.0.0.1:8901").unwrap(),
            edge_url: Url::parse("http://127.0.0.1:9080").unwrap(),
            timeout: Some(20),
            vin_match: 2,
        }
    }
}


impl Default for TestConfig {
    fn default() -> TestConfig {
        TestConfig {
            http: false,
            repl: false,
            websocket: true,
        }
    }
}

fn parse_toml(s: &str) -> Result<toml::Table, Error> {
    let mut parser = toml::Parser::new(&s);
    Ok(try!(parser.parse()
            .ok_or_else(move || parser.errors)))
}

fn parse_toml_table<T: Decodable>(tbl: &toml::Table, sect: &str) -> Result<T, Error> {

    let value = try!(tbl.get(sect)
                     .ok_or(Error::ParseError(format!(
                         "parse_toml_table, invalid section: {}", sect.to_string()))));

    let mut decoder = toml::Decoder::new(value.clone());

    Ok(try!(T::decode(&mut decoder)))

}

fn bootstrap_credentials(auth_cfg_section: AuthConfigSection) -> Result<AuthConfig, Error> {

    fn persist_credentials_file(creds: &CredentialsFile, path: &Path) -> Result<(), Error> {
        let mut tbl = toml::Table::new();
        tbl.insert("auth".to_string(), toml::encode(&creds));
        let dir = try!(path.parent()
                       .ok_or(Error::ParseError("Invalid credentials file path".to_string())));
        try!(fs::create_dir_all(&dir));
        let mut f = try!(File::create(path));
        try!(f.write_all(&toml::encode_str(&tbl).into_bytes()));
        Ok(())
    }

    fn read_credentials_file(mut f: File) -> Result<CredentialsFile, Error> {
        let mut s = String::new();
        try!(f.read_to_string(&mut s));
        let toml_table = try!(parse_toml(&s));
        let creds: CredentialsFile = try!(parse_toml_table(&toml_table, "auth"));
        Ok(creds)
    }

    let creds_path = Path::new(&auth_cfg_section.credentials_file);

    debug!("bootstrap_credentails: {:?}", creds_path);

    match File::open(creds_path) {
        Err(ref e) if e.kind() == ErrorKind::NotFound => {
            let creds = CredentialsFile { client_id: auth_cfg_section.client_id,
                                          secret: auth_cfg_section.secret,
                                          vin: auth_cfg_section.vin };
            try!(persist_credentials_file(&creds, &creds_path));
            Ok(AuthConfig::new(auth_cfg_section.server, creds))
        }
        Err(e)                                        => Err(Error::IoError(e)),
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

    Ok(Config {
        auth: auth_cfg,
        ota:  ota_cfg,
        test: test_cfg,
    })
}

pub fn load_config(path: &str) -> Result<Config, Error> {

    debug!("load_config: {}", path);

    match File::open(path) {
        Err(ref e) if e.kind() == ErrorKind::NotFound => Ok(Config::default()),
        Err(e)                                        => Err(Error::IoError(e)),
        Ok(mut f)                                     => {
            let mut s = String::new();
            try!(f.read_to_string(&mut s));
            parse_config(&s)
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
        vin = "V1234567890123456"

        [ota]
        server = "http://127.0.0.1:8080"
        polling_interval = 10
        packages_dir = "/tmp/"
        package_manager = "dpkg"

        [test]
        http = false
        repl = false
        websocket = true
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
