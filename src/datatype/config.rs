use rustc_serialize::Decodable;
use std::fs;
use std::fs::File;
use std::io::ErrorKind;
use std::os::unix::fs::PermissionsExt;
use std::io::prelude::*;
use std::path::Path;
use toml;
use toml::{Decoder, Parser, Table};

use datatype::{Error, SocketAddr, Url};
use package_manager::PackageManager;


/// A container for all parsed configs.
#[derive(Default, PartialEq, Eq, Debug, Clone)]
pub struct Config {
    pub auth:    Option<AuthConfig>,
    pub core:    CoreConfig,
    pub dbus:    Option<DBusConfig>,
    pub device:  DeviceConfig,
    pub gateway: GatewayConfig,
    pub network: NetworkConfig,
    pub rvi:     Option<RviConfig>,
}

impl Config {
    /// Read in a toml configuration file using default values for missing
    /// sections or fields.
    pub fn load(path: &str) -> Result<Config, Error> {
        info!("Loading config file: {}", path);
        let mut file = try!(File::open(path).map_err(Error::Io));
        let mut toml = String::new();
        try!(file.read_to_string(&mut toml));
        Config::parse(&toml)
    }

    /// Parse a toml configuration string using default values for missing
    /// sections or fields while retaining backwards compatibility.
    pub fn parse(toml: &str) -> Result<Config, Error> {
        let table = try!(parse_table(&toml));

        let mut auth:    Option<ParsedAuthConfig> = try!(maybe_parse_section(&table, "auth"));
        let mut core:    ParsedCoreConfig         = try!(parse_section(&table, "core"));
        let mut dbus:    Option<ParsedDBusConfig> = try!(maybe_parse_section(&table, "dbus"));
        let mut device:  ParsedDeviceConfig       = try!(parse_section(&table, "device"));
        let mut gateway: ParsedGatewayConfig      = try!(parse_section(&table, "gateway"));
        let mut network: ParsedNetworkConfig      = try!(parse_section(&table, "network"));
        let mut rvi:     Option<ParsedRviConfig>  = try!(maybe_parse_section(&table, "rvi"));

        if let Some(cfg) = auth {
            auth = Some(try!(bootstrap_credentials(cfg)));
        }

        try!(apply_transformations(&mut auth, &mut core, &mut dbus, &mut device,
                                   &mut gateway, &mut network, &mut rvi));

        Ok(Config {
            auth:    auth.map(|mut cfg| cfg.defaultify()),
            core:    core.defaultify(),
            dbus:    dbus.map(|mut cfg| cfg.defaultify()),
            device:  device.defaultify(),
            gateway: gateway.defaultify(),
            network: network.defaultify(),
            rvi:     rvi.map(|mut cfg| cfg.defaultify())
        })
    }
}

fn parse_table(toml: &str) -> Result<Table, Error> {
    let mut parser = Parser::new(toml);
    Ok(try!(parser.parse().ok_or_else(move || parser.errors)))
}

fn parse_section<T: Decodable + Default>(table: &Table, section: &str) -> Result<T, Error> {
    Ok(try!(maybe_parse_section(table, section)).unwrap_or(T::default()))
}

fn maybe_parse_section<T: Decodable>(table: &Table, section: &str) -> Result<Option<T>, Error> {
    table.get(section).map_or(Ok(None), |sect| {
        let mut decoder = Decoder::new(sect.clone());
        Ok(Some(try!(T::decode(&mut decoder))))
    })
}


#[derive(RustcEncodable, RustcDecodable)]
struct CredentialsFile {
    pub client_id:     String,
    pub client_secret: String,
}

// Read AuthConfig values from the credentials file if it exists, or write the
// current AuthConfig values to a new credentials file otherwise.
fn bootstrap_credentials(auth: ParsedAuthConfig) -> Result<ParsedAuthConfig, Error> {
    let creds = auth.credentials_file.expect("couldn't get credentials_file");
    let path  = Path::new(&creds);
    debug!("bootstrap_credentials: {:?}", path);

    let credentials = match File::open(path) {
        Ok(mut file) => {
            let mut text = String::new();
            try!(file.read_to_string(&mut text));
            let table = try!(parse_table(&text));
            let auth  = try!(table.get("auth").ok_or(Error::Config("no [auth] section".to_string())));
            let mut decoder = Decoder::new(auth.clone());
            try!(CredentialsFile::decode(&mut decoder))
        }

        Err(ref err) if err.kind() == ErrorKind::NotFound => {
            let mut table   = Table::new();
            let credentials = CredentialsFile {
                client_id:     auth.client_id.expect("expected client_id"),
                client_secret: auth.client_secret.expect("expected client_secret")
            };
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

    Ok(ParsedAuthConfig {
        server:           auth.server,
        client_id:        Some(credentials.client_id),
        client_secret:    Some(credentials.client_secret),
        credentials_file: Some(creds.clone()),
    })
}


// Apply transformations from old to new config fields for backwards compatibility.
fn apply_transformations(_:      &mut Option<ParsedAuthConfig>,
                         core:   &mut ParsedCoreConfig,
                         _:      &mut Option<ParsedDBusConfig>,
                         device: &mut ParsedDeviceConfig,
                         _:      &mut ParsedGatewayConfig,
                         _:      &mut ParsedNetworkConfig,
                         _:      &mut Option<ParsedRviConfig>) -> Result<(), Error> {

    match (device.polling_interval, core.polling_sec) {
        (Some(_), Some(_)) => {
            return Err(Error::Config("core.polling_sec and device.polling_interval both set".to_string()))
        }

        (Some(interval), None) => {
            if interval > 0 {
                core.polling     = Some(true);
                core.polling_sec = Some(interval);
            } else {
                core.polling = Some(false);
            }
        }

        _ => ()
    }

    Ok(())
}


/// Trait used to overwrite any `None` fields in a config with its default value.
trait Defaultify<T: Default> {
    fn defaultify(&mut self) -> T;
}


/// The [auth] configuration section.
#[derive(RustcDecodable, PartialEq, Eq, Debug, Clone)]
pub struct AuthConfig {
    pub server:           Url,
    pub client_id:        String,
    pub client_secret:    String,
    pub credentials_file: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        AuthConfig {
            server:           "http://127.0.0.1:9001".parse().unwrap(),
            client_id:        "client-id".to_string(),
            client_secret:    "client-secret".to_string(),
            credentials_file: "/tmp/sota_credentials.toml".to_string(),
        }
    }
}

#[derive(RustcDecodable)]
struct ParsedAuthConfig {
    server:           Option<Url>,
    client_id:        Option<String>,
    client_secret:    Option<String>,
    credentials_file: Option<String>,
}

impl Default for ParsedAuthConfig {
    fn default() -> Self {
        ParsedAuthConfig {
            server:           None,
            client_id:        None,
            client_secret:    None,
            credentials_file: None
        }
    }
}

impl Defaultify<AuthConfig> for ParsedAuthConfig {
    fn defaultify(&mut self) -> AuthConfig {
        let default = AuthConfig::default();
        AuthConfig {
            server:           self.server.take().unwrap_or(default.server),
            client_id:        self.client_id.take().unwrap_or(default.client_id),
            client_secret:    self.client_secret.take().unwrap_or(default.client_secret),
            credentials_file: self.credentials_file.take().unwrap_or(default.credentials_file)
        }
    }
}


/// The [core] configuration section.
#[derive(RustcDecodable, PartialEq, Eq, Debug, Clone)]
pub struct CoreConfig {
    pub server:      Url,
    pub polling:     bool,
    pub polling_sec: u64
}

impl Default for CoreConfig {
    fn default() -> CoreConfig {
        CoreConfig {
            server:      "http://127.0.0.1:8080".parse().unwrap(),
            polling:     true,
            polling_sec: 10
        }
    }
}

#[derive(RustcDecodable)]
struct ParsedCoreConfig {
    server:      Option<Url>,
    polling:     Option<bool>,
    polling_sec: Option<u64>
}

impl Default for ParsedCoreConfig {
    fn default() -> Self {
        ParsedCoreConfig {
            server:      None,
            polling:     None,
            polling_sec: None
        }
    }
}

impl Defaultify<CoreConfig> for ParsedCoreConfig {
    fn defaultify(&mut self) -> CoreConfig {
        let default = CoreConfig::default();
        CoreConfig {
            server:      self.server.take().unwrap_or(default.server),
            polling:     self.polling.take().unwrap_or(default.polling),
            polling_sec: self.polling_sec.take().unwrap_or(default.polling_sec)
        }
    }
}


/// The [dbus] configuration section.
#[derive(RustcDecodable, PartialEq, Eq, Debug, Clone)]
pub struct DBusConfig {
    pub name:                  String,
    pub path:                  String,
    pub interface:             String,
    pub software_manager:      String,
    pub software_manager_path: String,
    pub timeout:               i32,
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

#[derive(RustcDecodable)]
struct ParsedDBusConfig {
    name:                  Option<String>,
    path:                  Option<String>,
    interface:             Option<String>,
    software_manager:      Option<String>,
    software_manager_path: Option<String>,
    timeout:               Option<i32>,
}

impl Default for ParsedDBusConfig {
    fn default() -> Self {
        ParsedDBusConfig {
            name:                  None,
            path:                  None,
            interface:             None,
            software_manager:      None,
            software_manager_path: None,
            timeout:               None
        }
    }
}

impl Defaultify<DBusConfig> for ParsedDBusConfig {
    fn defaultify(&mut self) -> DBusConfig {
        let default = DBusConfig::default();
        DBusConfig {
            name:                  self.name.take().unwrap_or(default.name),
            path:                  self.path.take().unwrap_or(default.path),
            interface:             self.interface.take().unwrap_or(default.interface),
            software_manager:      self.software_manager.take().unwrap_or(default.software_manager),
            software_manager_path: self.software_manager_path.take().unwrap_or(default.software_manager_path),
            timeout:               self.timeout.take().unwrap_or(default.timeout)
        }
    }
}


/// The [device] configuration section.
#[derive(RustcDecodable, PartialEq, Eq, Debug, Clone)]
pub struct DeviceConfig {
    pub uuid:              String,
    pub vin:               String,
    pub packages_dir:      String,
    pub package_manager:   PackageManager,
    pub certificates_path: String,
    pub system_info:       Option<String>,
}

impl Default for DeviceConfig {
    fn default() -> DeviceConfig {
        DeviceConfig {
            uuid:              "123e4567-e89b-12d3-a456-426655440000".to_string(),
            vin:               "V1234567890123456".to_string(),
            packages_dir:      "/tmp/".to_string(),
            package_manager:   PackageManager::Off,
            certificates_path: "/tmp/sota_certificates".to_string(),
            system_info:       Some("system_info.sh".to_string())
        }
    }
}

#[derive(RustcDecodable)]
struct ParsedDeviceConfig {
    pub uuid:              Option<String>,
    pub vin:               Option<String>,
    pub packages_dir:      Option<String>,
    pub package_manager:   Option<PackageManager>,
    pub polling_interval:  Option<u64>,
    pub certificates_path: Option<String>,
    pub system_info:       Option<String>,
}

impl Default for ParsedDeviceConfig {
    fn default() -> Self {
        ParsedDeviceConfig {
            uuid:              None,
            vin:               None,
            packages_dir:      None,
            package_manager:   None,
            polling_interval:  None,
            certificates_path: None,
            system_info:       None,
        }
    }
}

impl Defaultify<DeviceConfig> for ParsedDeviceConfig {
    fn defaultify(&mut self) -> DeviceConfig {
        let default = DeviceConfig::default();
        DeviceConfig {
            uuid:              self.uuid.take().unwrap_or(default.uuid),
            vin:               self.vin.take().unwrap_or(default.vin),
            packages_dir:      self.packages_dir.take().unwrap_or(default.packages_dir),
            package_manager:   self.package_manager.take().unwrap_or(default.package_manager),
            certificates_path: self.certificates_path.take().unwrap_or(default.certificates_path),
            system_info:       self.system_info.take().or(default.system_info),
        }
    }
}


/// The [gateway] configuration section.
#[derive(RustcDecodable, PartialEq, Eq, Debug, Clone)]
pub struct GatewayConfig {
    pub console:   bool,
    pub dbus:      bool,
    pub http:      bool,
    pub rvi:       bool,
    pub socket:    bool,
    pub websocket: bool,
}

impl Default for GatewayConfig {
    fn default() -> GatewayConfig {
        GatewayConfig {
            console:   false,
            dbus:      false,
            http:      false,
            rvi:       false,
            socket:    false,
            websocket: false,
        }
    }
}

#[derive(RustcDecodable)]
struct ParsedGatewayConfig {
    console:   Option<bool>,
    dbus:      Option<bool>,
    http:      Option<bool>,
    rvi:       Option<bool>,
    socket:    Option<bool>,
    websocket: Option<bool>,
}

impl Default for ParsedGatewayConfig {
    fn default() -> Self {
        ParsedGatewayConfig {
            console:   None,
            dbus:      None,
            http:      None,
            rvi:       None,
            socket:    None,
            websocket: None
        }
    }
}

impl Defaultify<GatewayConfig> for ParsedGatewayConfig {
    fn defaultify(&mut self) -> GatewayConfig {
        let default = GatewayConfig::default();
        GatewayConfig {
            console:   self.console.take().unwrap_or(default.console),
            dbus:      self.dbus.take().unwrap_or(default.dbus),
            http:      self.http.take().unwrap_or(default.http),
            rvi:       self.rvi.take().unwrap_or(default.rvi),
            socket:    self.socket.take().unwrap_or(default.socket),
            websocket: self.websocket.take().unwrap_or(default.websocket)
        }
    }
}


/// The [network] configuration section.
#[derive(RustcDecodable, PartialEq, Eq, Debug, Clone)]
pub struct NetworkConfig {
    pub http_server:          SocketAddr,
    pub rvi_edge_server:      SocketAddr,
    pub socket_commands_path: String,
    pub socket_events_path:   String,
    pub websocket_server:     String
}

impl Default for NetworkConfig {
    fn default() -> NetworkConfig {
        NetworkConfig {
            http_server:          "127.0.0.1:8888".parse().unwrap(),
            rvi_edge_server:      "127.0.0.1:9080".parse().unwrap(),
            socket_commands_path: "/tmp/sota-commands.socket".to_string(),
            socket_events_path:   "/tmp/sota-events.socket".to_string(),
            websocket_server:     "127.0.0.1:3012".to_string()
        }
    }
}

#[derive(RustcDecodable)]
struct ParsedNetworkConfig {
    http_server:          Option<SocketAddr>,
    rvi_edge_server:      Option<SocketAddr>,
    socket_commands_path: Option<String>,
    socket_events_path:   Option<String>,
    websocket_server:     Option<String>
}

impl Default for ParsedNetworkConfig {
    fn default() -> Self {
        ParsedNetworkConfig {
            http_server:          None,
            rvi_edge_server:      None,
            socket_commands_path: None,
            socket_events_path:   None,
            websocket_server:     None
        }
    }
}

impl Defaultify<NetworkConfig> for ParsedNetworkConfig {
    fn defaultify(&mut self) -> NetworkConfig {
        let default = NetworkConfig::default();
        NetworkConfig {
            http_server:          self.http_server.take().unwrap_or(default.http_server),
            rvi_edge_server:      self.rvi_edge_server.take().unwrap_or(default.rvi_edge_server),
            socket_commands_path: self.socket_commands_path.take().unwrap_or(default.socket_commands_path),
            socket_events_path:   self.socket_events_path.take().unwrap_or(default.socket_events_path),
            websocket_server:     self.websocket_server.take().unwrap_or(default.websocket_server)
        }
    }
}


/// The [rvi] configuration section.
#[derive(RustcDecodable, PartialEq, Eq, Debug, Clone)]
pub struct RviConfig {
    pub client:      Url,
    pub storage_dir: String,
    pub timeout:     Option<i64>,
}

impl Default for RviConfig {
    fn default() -> RviConfig {
        RviConfig {
            client:      "http://127.0.0.1:8901".parse().unwrap(),
            storage_dir: "/var/sota".to_string(),
            timeout:     None,
        }
    }
}

#[derive(RustcDecodable)]
struct ParsedRviConfig {
    client:      Option<Url>,
    storage_dir: Option<String>,
    timeout:     Option<i64>,
}

impl Default for ParsedRviConfig {
    fn default() -> Self {
        ParsedRviConfig {
            client:      None,
            storage_dir: None,
            timeout:     None
        }
    }
}

impl Defaultify<RviConfig> for ParsedRviConfig {
    fn defaultify(&mut self) -> RviConfig {
        let default = RviConfig::default();
        RviConfig {
            client:      self.client.take().unwrap_or(default.client),
            storage_dir: self.storage_dir.take().unwrap_or(default.storage_dir),
            timeout:     self.timeout.take().or(default.timeout)
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
        client_secret = "client-secret"
        credentials_file = "/tmp/sota_credentials.toml"
        "#;

    const CORE_CONFIG: &'static str =
        r#"
        [core]
        server = "http://127.0.0.1:8080"
        polling = true
        polling_sec = 10
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
        packages_dir = "/tmp/"
        package_manager = "off"
        certificates_path = "/tmp/sota_certificates"
        system_info = "system_info.sh"
        "#;

    const GATEWAY_CONFIG: &'static str =
        r#"
        [gateway]
        console = false
        dbus = false
        http = false
        rvi = false
        socket = false
        websocket = false
        "#;

    const NETWORK_CONFIG: &'static str =
        r#"
        [network]
        http_server = "127.0.0.1:8888"
        rvi_edge_server = "127.0.0.1:9080"
        socket_commands_path = "/tmp/sota-commands.socket"
        socket_events_path = "/tmp/sota-events.socket"
        websocket_server = "127.0.0.1:3012"
        "#;

    const RVI_CONFIG: &'static str =
        r#"
        [rvi]
        client = "http://127.0.0.1:8901"
        storage_dir = "/var/sota"
        timeout = 20
        "#;


    #[test]
    fn empty_config() {
        assert_eq!(Config::parse("").unwrap(), Config::default());
    }

    #[test]
    fn basic_config() {
        let config = String::new()
            + CORE_CONFIG
            + DEVICE_CONFIG
            + GATEWAY_CONFIG
            + NETWORK_CONFIG;
        assert_eq!(Config::parse(&config).unwrap(), Config::default());
    }

    #[test]
    fn default_config() {
        let config = String::new()
            + AUTH_CONFIG
            + CORE_CONFIG
            + DBUS_CONFIG
            + DEVICE_CONFIG
            + GATEWAY_CONFIG
            + NETWORK_CONFIG
            + RVI_CONFIG;
        assert_eq!(Config::load("tests/toml/default.toml").unwrap(), Config::parse(&config).unwrap());
    }

    #[test]
    fn backwards_compatible_config() {
        let config = Config::load("tests/toml/old.toml").unwrap();
        assert_eq!(config.core.polling, true);
        assert_eq!(config.core.polling_sec, 10);
    }
}
