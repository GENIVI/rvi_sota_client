// TODO: maybe break apart into submodules, one for each subgroup
use toml;
use std::io::prelude::*;
use std::path::PathBuf;
use std::fs::OpenOptions;
use std::env;
use std::result;
use std::fmt;

pub type Result<T> = result::Result<T, String>;

pub struct Configuration {
    pub client: ClientConfiguration,
    pub dbus: DBusConfiguration
}

impl Configuration {
    pub fn read(file: &str) -> Result<Configuration> {
        let path = PathBuf::from(file);
        let mut f = try!(OpenOptions::new().open(path).map_err(stringify));
        let mut buf = Vec::new();
        try!(f.read_to_end(&mut buf).map_err(stringify));
        let data = try!(String::from_utf8(buf).map_err(stringify));
        Configuration::parse(&data)
    }

    pub fn parse(conf: &str) -> Result<Configuration> {
        let mut parser = toml::Parser::new(conf);
        let tree = try!(parser.parse().ok_or(format_parser_error(&parser)));

        let client = try!(ClientConfiguration::parse(&tree));
        let dbus   = try!(DBusConfiguration::parse(&tree));

        Ok(Configuration {
            client: client,
            dbus: dbus
        })
    }

    pub fn default_path() -> String {
        match env::var_os("XDG_CONFIG_HOME")
            .and_then(|s| s.into_string().ok()) {
            Some(val) => { return val + "/sota/client.toml"; },
            None => { error!("$XDG_CONFIG_HOME is not set"); }
        }

        match env::var_os("HOME").and_then(|s| s.into_string().ok()) {
            Some(val) => {
                warn!("Falling back to $HOME/.config");
                return val + "/.sota/client.toml";
            },
            None => { error!("$HOME is not set"); }
        }

        warn!("Falling back to $PWD");
        ".sota/client.toml".to_string()
    }
}

trait ConfTreeParser<C> {
    fn parse(tree: &toml::Table) -> Result<C>;
}

pub struct ClientConfiguration {
    pub storage_dir: String,
    pub rvi_url: Option<String>,
    pub edge_url: Option<String>
}

impl ConfTreeParser<ClientConfiguration> for ClientConfiguration {
    fn parse(tree: &toml::Table) -> Result<ClientConfiguration> {
        let client_tree = try!(tree.get("client")
            .ok_or("Missing required subgroup \"client\""));

        let storage_dir = try!(get_required_key(client_tree, "storage_dir", "client"));
        let rvi_url = try!(get_optional_key(client_tree, "rvi_url", "client"));
        let edge_url = try!(get_optional_key(client_tree, "edge_url", "client"));

        Ok(ClientConfiguration {
            storage_dir: storage_dir,
            rvi_url: rvi_url,
            edge_url: edge_url
        })
    }
}

#[derive(Clone)]
pub struct DBusConfiguration {
    pub name: String,
    pub interface: String,
    pub software_manager: String,
    pub timeout: i32 // dbus-rs expects a signed int
}

impl DBusConfiguration {
    #[cfg(test)]
    pub fn gen_test() -> DBusConfiguration {
        DBusConfiguration {
            name: "org.test.test".to_string(),
            interface: "org.test.test".to_string(),
            software_manager: "org.test.software_manager".to_string(),
            timeout: 5000
        }
    }
}

impl ConfTreeParser<DBusConfiguration> for DBusConfiguration {
    fn parse(tree: &toml::Table) -> Result<DBusConfiguration> {
        let dbus_tree = try!(tree.get("dbus")
                             .ok_or("Missing required subgroup \"dbus\""));
        let name = try!(get_required_key(dbus_tree, "name", "dbus"));
        let interface = try!(get_required_key(dbus_tree, "interface", "dbus"));
        let software_manager = try!(get_required_key(dbus_tree,
                                                     "software_manager",
                                                     "dbus"));
        let timeout = try!(get_optional_key(dbus_tree, "timeout", "dbus"));

        Ok(DBusConfiguration {
            name: name,
            interface: interface,
            software_manager: software_manager,
            timeout: timeout.unwrap_or(10000)
        })
    }
}

fn get_required_key<D>(subtree: &toml::Value, key: &str, group: &str)
    -> Result<D> where D: ParseTomlValue {
    let value = try!(subtree.lookup(key)
                     .ok_or(format!("Missing required key \"{}\" in \"{}\"",
                                    key, group)));
    ParseTomlValue::parse(value, key, group)
}

// This basically does a Option<Result> -> Result<Option> translation
fn get_optional_key<D>(subtree: &toml::Value, key: &str, group: &str)
    -> Result<Option<D>> where D: ParseTomlValue {
    match subtree.lookup(key) {
        Some(val) => {
            Ok(Some(try!(ParseTomlValue::parse(val, key, group))))
        },
        None => Ok(None)
    }
}

trait ParseTomlValue {
    fn parse(val: &toml::Value, key: &str, group: &str) -> Result<Self>;
}

impl ParseTomlValue for String {
    fn parse(val: &toml::Value, key: &str, group: &str)
        -> Result<String> {
        val.as_str().map(|s| s.to_string())
            .ok_or(format!("Key \"{}\" in \"{}\" is not a string", key, group))
    }
}

impl ParseTomlValue for i32 {
    fn parse(val: &toml::Value, key: &str, group: &str)
        -> Result<i32> {
        val.as_integer().map(|i| i as i32)
            .ok_or(format!("Key \"{}\" in \"{}\" is not a integer", key, group))
    }
}

#[cfg(not(test))]
fn format_parser_error(parser: &toml::Parser) -> String {
    let linecol = parser.to_linecol(0);
    format!("parse error: {}:{}: {:?}", linecol.0, linecol.1, parser.errors)
}

#[cfg(test)]
fn format_parser_error(parser: &toml::Parser) -> String {
    format!("parse error: {:?}", parser.errors)
}

fn stringify<T>(e: T) -> String
    where T: fmt::Display {
    format!("{}", e)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::env;

    static STORAGE: &'static str = "/var/sota";
    static RVI: &'static str = "/http://localhost:8901";
    static EDGE: &'static str = "localhost:9080";

    static NAME: &'static str = "org.genivi.sota_client";
    static INTERFACE: &'static str = "org.genivi.software_manager";
    static SOFTWARE_MANAGER: &'static str = "org.genivi.software_manager";

    #[test]
    fn it_correctly_parses_a_valid_configuration() {
        test_init!();
        let data = format!(r#"
        [client]
        storage_dir = "{}"
        rvi_url = "{}"
        edge_url = "{}"

        [dbus]
        name = "{}"
        interface = "{}"
        software_manager = "{}"
        "#, STORAGE, RVI, EDGE,
        NAME, INTERFACE, SOFTWARE_MANAGER);

        let configuration = Configuration::parse(&data).unwrap();
        assert_eq!(&configuration.client.storage_dir, STORAGE);
        assert_eq!(&configuration.client.rvi_url.unwrap(), RVI);
        assert_eq!(&configuration.client.edge_url.unwrap(), EDGE);

        assert_eq!(&configuration.dbus.name, NAME);
        assert_eq!(&configuration.dbus.interface, INTERFACE);
        assert_eq!(&configuration.dbus.software_manager, SOFTWARE_MANAGER);
    }

    #[test]
    fn it_ignores_extra_keys() {
        test_init!();
        let data = format!(r#"
        [client]
        storage_dir = "{}"
        rvi_url = "{}"
        edge_url = "{}"
        test_key = "hello world"

        [dbus]
        name = "{}"
        interface = "{}"
        software_manager = "{}"
        test_key = "see ya world"
        "#, STORAGE, RVI, EDGE,
        NAME, INTERFACE, SOFTWARE_MANAGER);

        let configuration = Configuration::parse(&data).unwrap();
        assert_eq!(&configuration.client.storage_dir, STORAGE);
        assert_eq!(&configuration.client.rvi_url.unwrap(), RVI);
        assert_eq!(&configuration.client.edge_url.unwrap(), EDGE);

        assert_eq!(&configuration.dbus.name, NAME);
        assert_eq!(&configuration.dbus.interface, INTERFACE);
        assert_eq!(&configuration.dbus.software_manager, SOFTWARE_MANAGER);
    }

    #[test]
    fn it_ignores_extra_groups() {
        test_init!();
        let data = format!(r#"
        [client]
        storage_dir = "{}"
        rvi_url = "{}"
        edge_url = "{}"

        [dbus]
        name = "{}"
        interface = "{}"
        software_manager = "{}"

        [testgroup]
        is_empty = true
        "#, STORAGE, RVI, EDGE,
        NAME, INTERFACE, SOFTWARE_MANAGER);

        let configuration = Configuration::parse(&data).unwrap();
        assert_eq!(&configuration.client.storage_dir, STORAGE);
        assert_eq!(&configuration.client.rvi_url.unwrap(), RVI);
        assert_eq!(&configuration.client.edge_url.unwrap(), EDGE);

        assert_eq!(&configuration.dbus.name, NAME);
        assert_eq!(&configuration.dbus.interface, INTERFACE);
        assert_eq!(&configuration.dbus.software_manager, SOFTWARE_MANAGER);
    }

    #[test]
    fn it_requires_the_client_group() {
        test_init!();
        let data = format!(r#"
        [dbus]
        name = "{}"
        interface = "{}"
        software_manager = "{}"
        "#, NAME, INTERFACE, SOFTWARE_MANAGER);
        match Configuration::parse(&data) {
            Ok(..) => panic!("Accepted invalid configuration!"),
            Err(e) => {
                assert_eq!(e, "Missing required subgroup \"client\""
                              .to_string());
            }
        };
    }

    #[test]
    fn it_requires_the_dbus_group() {
        test_init!();
        let data = format!(r#"
        [client]
        storage_dir = "{}"
        rvi_url = "{}"
        edge_url = "{}"
        "#, STORAGE, RVI, EDGE);
        match Configuration::parse(&data) {
            Ok(..) => panic!("Accepted invalid configuration!"),
            Err(e) => {
                assert_eq!(e, "Missing required subgroup \"dbus\""
                              .to_string());
            }
        };
    }

    #[test]
    fn it_requires_the_storage_dir_key() {
        test_init!();
        let data = format!(r#"
        [client]
        rvi_url = "{}"
        edge_url = "{}"

        [dbus]
        name = "{}"
        interface = "{}"
        software_manager = "{}"
        "#, RVI, EDGE,
        NAME, INTERFACE, SOFTWARE_MANAGER);

        match Configuration::parse(&data) {
            Ok(..) => panic!("Accepted invalid configuration!"),
            Err(e) => {
                assert_eq!(e,
                           "Missing required key \"storage_dir\" in \"client\""
                           .to_string());
            }
        };
    }

    #[test]
    fn it_doesnt_require_the_rvi_url_key() {
        test_init!();
        let data = format!(r#"
        [client]
        storage_dir = "{}"
        edge_url = "{}"

        [dbus]
        name = "{}"
        interface = "{}"
        software_manager = "{}"
        "#, STORAGE, EDGE,
        NAME, INTERFACE, SOFTWARE_MANAGER);

        let configuration = Configuration::parse(&data).unwrap();
        assert_eq!(&configuration.client.storage_dir, STORAGE);
        assert_eq!(configuration.client.rvi_url, None);
        assert_eq!(&configuration.client.edge_url.unwrap(), EDGE);

        assert_eq!(&configuration.dbus.name, NAME);
        assert_eq!(&configuration.dbus.interface, INTERFACE);
        assert_eq!(&configuration.dbus.software_manager, SOFTWARE_MANAGER);
    }

    #[test]
    fn it_doesnt_require_the_edge_url_key() {
        test_init!();
        let data = format!(r#"
        [client]
        storage_dir = "{}"
        rvi_url = "{}"

        [dbus]
        name = "{}"
        interface = "{}"
        software_manager = "{}"
        "#, STORAGE, RVI,
        NAME, INTERFACE, SOFTWARE_MANAGER);

        let configuration = Configuration::parse(&data).unwrap();
        assert_eq!(&configuration.client.storage_dir, STORAGE);
        assert_eq!(&configuration.client.rvi_url.unwrap(), RVI);
        assert_eq!(configuration.client.edge_url, None);

        assert_eq!(&configuration.dbus.name, NAME);
        assert_eq!(&configuration.dbus.interface, INTERFACE);
        assert_eq!(&configuration.dbus.software_manager, SOFTWARE_MANAGER);
    }

    #[test]
    fn it_requires_the_dbus_name_key() {
        test_init!();
        let data = format!(r#"
        [client]
        storage_dir = "{}"
        rvi_url = "{}"
        edge_url = "{}"

        [dbus]
        interface = "{}"
        software_manager = "{}"
        "#, STORAGE, RVI, EDGE,
        INTERFACE, SOFTWARE_MANAGER);

        match Configuration::parse(&data) {
            Ok(..) => panic!("Accepted invalid configuration!"),
            Err(e) => {
                assert_eq!(e,
                           "Missing required key \"name\" in \"dbus\""
                           .to_string());
            }
        };
    }

    #[test]
    fn it_requires_the_dbus_interface_key() {
        test_init!();
        let data = format!(r#"
        [client]
        storage_dir = "{}"
        rvi_url = "{}"
        edge_url = "{}"

        [dbus]
        name = "{}"
        software_manager = "{}"
        "#, STORAGE, RVI, EDGE,
        NAME, SOFTWARE_MANAGER);

        match Configuration::parse(&data) {
            Ok(..) => panic!("Accepted invalid configuration!"),
            Err(e) => {
                assert_eq!(e,
                           "Missing required key \"interface\" in \"dbus\""
                           .to_string());
            }
        };
    }

    #[test]
    fn it_requires_the_dbus_software_manager_key() {
        test_init!();
        let data = format!(r#"
        [client]
        storage_dir = "{}"
        rvi_url = "{}"
        edge_url = "{}"

        [dbus]
        name = "{}"
        interface = "{}"
        "#, STORAGE, RVI, EDGE,
        NAME, INTERFACE);

        match Configuration::parse(&data) {
            Ok(..) => panic!("Accepted invalid configuration!"),
            Err(e) => {
                assert_eq!(e, "Missing required key \"software_manager\" \
                           in \"dbus\"".to_string());
            }
        };
    }

    #[test]
    fn it_uses_fallbacks_for_its_configuration() {
        test_init!();
        env::remove_var("XDG_CONFIG_HOME");
        env::set_var("XDG_CONFIG_HOME", "/some/thing");
        assert_eq!(Configuration::default_path(),
                   "/some/thing/sota/client.toml".to_string());
        env::remove_var("XDG_CONFIG_HOME");
        env::remove_var("HOME");
        env::set_var("HOME", "/some/thing");
        assert_eq!(Configuration::default_path(),
                   "/some/thing/.sota/client.toml".to_string());
        env::remove_var("XDG_CONFIG_HOME");
        env::remove_var("HOME");
        assert_eq!(Configuration::default_path(),
                   ".sota/client.toml".to_string());
    }
}
