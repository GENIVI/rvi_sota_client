use rustc_serialize::Decodable;
use std::fs;
use std::fs::File;
use std::io::ErrorKind;
use std::os::unix::fs::PermissionsExt;
use std::io::prelude::*;
use std::path::Path;
use toml;
use toml::{Decoder, Parser, Table};

use datatype::{Error, Url};
use package_manager::PackageManager;


#[derive(Default, PartialEq, Eq, Debug, Clone)]
pub struct Config {
    pub auth:    Option<AuthConfig>,
    pub dbus:    DBusConfig,
    pub device:  DeviceConfig,
    pub gateway: GatewayConfig,
    pub core:    CoreConfig,
    pub rvi:     RviConfig,
}

pub fn load_config(path: &str) -> Result<Config, Error> {
    debug!("load_config: {}", path);
    match File::open(path) {
        Ok(mut file) => {
            let mut text = String::new();
            try!(file.read_to_string(&mut text));
            parse_config(&text)
        }

        Err(ref err) if err.kind() == ErrorKind::NotFound => {
            error!("config file {} not found; using default config...", path);
            Ok(Config::default())
        }

        Err(err) => Err(Error::Io(err)),
    }
}

pub fn parse_config(toml: &str) -> Result<Config, Error> {
    let table = try!(parse_table(toml));

    let auth_cfg = if table.contains_key("auth") {
        let parsed: AuthConfig = try!(parse_section(&table, "auth"));
        Some(try!(bootstrap_credentials(parsed)))
    } else {
        None
    };

    Ok(Config {
        auth:    auth_cfg,
        core:    try!(parse_section(&table, "core")),
        dbus:    try!(parse_section(&table, "dbus")),
        device:  try!(parse_section(&table, "device")),
        rvi:     try!(parse_section(&table, "rvi")),
        gateway: try!(parse_section(&table, "gateway")),
    })
}

fn parse_table(toml: &str) -> Result<Table, Error> {
    let mut parser = Parser::new(toml);
    Ok(try!(parser.parse().ok_or_else(move || parser.errors)))
}

fn parse_section<T: Decodable>(table: &Table, section: &str) -> Result<T, Error> {
    let section = try!(table.get(section).ok_or_else(|| {
        Error::Parse(format!("invalid section: {}", section.to_string()))
    }));
    let mut decoder = Decoder::new(section.clone());
    Ok(try!(T::decode(&mut decoder)))
}


#[derive(RustcEncodable, RustcDecodable)]
struct CredentialsFile {
    pub client_id: String,
    pub secret:    String,
}

// Read AuthConfig values from the credentials file if it exists, or write the
// current AuthConfig values to a new credentials file otherwise.
fn bootstrap_credentials(auth_cfg: AuthConfig) -> Result<AuthConfig, Error> {
    let creds = auth_cfg.credentials_file.clone();
    let path = Path::new(&creds);
    debug!("bootstrap_credentials: {:?}", path);

    let credentials = match File::open(path) {
        Ok(mut file) => {
            let mut text = String::new();
            try!(file.read_to_string(&mut text));
            let table = try!(parse_table(&text));
            try!(parse_section::<CredentialsFile>(&table, "auth"))
        }

        Err(ref err) if err.kind() == ErrorKind::NotFound => {
            let mut table   = Table::new();
            let credentials = CredentialsFile { client_id: auth_cfg.client_id, secret: auth_cfg.secret };
            table.insert("auth".to_string(), toml::encode(&credentials));

            let dir = try!(path.parent().ok_or(Error::Parse("Invalid credentials file path".to_string())));
            try!(fs::create_dir_all(&dir));
            let mut file  = try!(File::create(path));
            let mut perms = try!(file.metadata()).permissions();
            perms.set_mode(0o600);
            try!(fs::set_permissions(path, perms));
            try!(file.write_all(&toml::encode_str(&table).into_bytes()));

            credentials
        }

        Err(err) => return Err(Error::Io(err))
    };

    Ok(AuthConfig {
        server:           auth_cfg.server,
        client_id:        credentials.client_id,
        secret:           credentials.secret,
        credentials_file: auth_cfg.credentials_file,
    })
}


#[derive(RustcDecodable, PartialEq, Eq, Debug, Clone)]
pub struct AuthConfig {
    pub server:           Url,
    pub client_id:        String,
    pub secret:           String,
    pub credentials_file: String,
}

impl Default for AuthConfig {
    fn default() -> AuthConfig {
        AuthConfig {
            server:           "http://127.0.0.1:9001".parse().unwrap(),
            client_id:        "client-id".to_string(),
            secret:           "secret".to_string(),
            credentials_file: "/tmp/sota_credentials.toml".to_string(),
        }
    }
}


#[derive(RustcDecodable, PartialEq, Eq, Debug, Clone)]
pub struct CoreConfig {
    pub server: Url
}

impl Default for CoreConfig {
    fn default() -> CoreConfig {
        CoreConfig {
            server: "http://127.0.0.1:8080".parse().unwrap()
        }
    }
}


#[derive(RustcDecodable, PartialEq, Eq, Debug, Clone)]
pub struct DBusConfig {
    pub name:                  String,
    pub path:                  String,
    pub interface:             String,
    pub software_manager:      String,
    pub software_manager_path: String,
    pub timeout:               i32, // dbus-rs expects a signed int
}

impl Default for DBusConfig {
    fn default() -> DBusConfig {
        DBusConfig {
            name:                  "org.genivi.SotaClient".to_string(),
            path:                  "/org/genivi/SotaClient".to_string(),
            interface:             "org.genivi.SotaClient".to_string(),
            software_manager:      "org.genivi.SoftwareLoadingManager".to_string(),
            software_manager_path: "/org/genivi/SoftwareLoadingManager".to_string(),
            timeout:               60
        }
    }
}


#[derive(RustcDecodable, PartialEq, Eq, Debug, Clone)]
pub struct DeviceConfig {
    pub uuid:              String,
    pub vin:               String,
    pub packages_dir:      String,
    pub package_manager:   PackageManager,
    pub polling_interval:  u64,
    pub certificates_path: String,
}

impl Default for DeviceConfig {
    fn default() -> DeviceConfig {
        DeviceConfig {
            uuid:              "123e4567-e89b-12d3-a456-426655440000".to_string(),
            vin:               "V1234567890123456".to_string(),
            packages_dir:      "/tmp/".to_string(),
            package_manager:   PackageManager::Dpkg,
            polling_interval:  10,
            certificates_path: "/tmp/sota_certificates".to_string()
        }
    }
}


#[derive(RustcDecodable, PartialEq, Eq, Debug, Clone)]
pub struct GatewayConfig {
    pub console:   bool,
    pub dbus:      bool,
    pub http:      bool,
    pub websocket: bool,
}

impl Default for GatewayConfig {
    fn default() -> GatewayConfig {
        GatewayConfig {
            console:   false,
            dbus:      false,
            http:      false,
            websocket: true,
        }
    }
}


#[derive(RustcDecodable, PartialEq, Eq, Debug, Clone)]
pub struct RviConfig {
    pub client:      Url,
    pub edge:        Url,
    pub storage_dir: String,
    pub timeout:     Option<i64>,
}

impl Default for RviConfig {
    fn default() -> RviConfig {
        RviConfig {
            client:      "http://127.0.0.1:8901".parse().unwrap(),
            edge:        "http://127.0.0.1:9080".parse().unwrap(),
            storage_dir: "/var/sota".to_string(),
            timeout:     Some(20),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;


    const AUTH_CONFIG: &'static str =
        r#"
        [auth]
        server = "http://127.0.0.1:9001"
        client_id = "client-id"
        secret = "secret"
        credentials_file = "/tmp/sota_credentials.toml"
        "#;

    const CORE_CONFIG: &'static str =
        r#"
        [core]
        server = "http://127.0.0.1:8080"
        "#;

    const DBUS_CONFIG: &'static str =
        r#"
        [dbus]
        name = "org.genivi.SotaClient"
        path = "/org/genivi/SotaClient"
        interface = "org.genivi.SotaClient"
        software_manager = "org.genivi.SoftwareLoadingManager"
        software_manager_path = "/org/genivi/SoftwareLoadingManager"
        timeout = 60
        "#;

    const DEVICE_CONFIG: &'static str =
        r#"
        [device]
        uuid = "123e4567-e89b-12d3-a456-426655440000"
        vin = "V1234567890123456"
        polling_interval = 10
        packages_dir = "/tmp/"
        package_manager = "dpkg"
        certificates_path = "/tmp/sota_certificates"
        "#;

    const GATEWAY_CONFIG: &'static str =
        r#"
        [gateway]
        console = false
        dbus = false
        http = false
        websocket = true
        "#;

    const RVI_CONFIG: &'static str =
        r#"
        [rvi]
        client = "http://127.0.0.1:8901"
        edge = "http://127.0.0.1:9080"
        storage_dir = "/var/sota"
        timeout = 20
        "#;


    #[test]
    fn parse_default_config() {
        let config = String::new()
            + CORE_CONFIG
            + DBUS_CONFIG
            + DEVICE_CONFIG
            + GATEWAY_CONFIG
            + RVI_CONFIG;
        assert_eq!(parse_config(&config).unwrap(), Config::default());
    }

    #[test]
    fn parse_example_config() {
        let config = String::new()
            + AUTH_CONFIG
            + CORE_CONFIG
            + DBUS_CONFIG
            + DEVICE_CONFIG
            + GATEWAY_CONFIG
            + RVI_CONFIG;
        assert_eq!(load_config("tests/sota.toml").unwrap(), parse_config(&config).unwrap());
    }

    #[test]
    fn bad_path_yields_default_config() {
        assert_eq!(load_config("").unwrap(), Config::default())
    }
}
