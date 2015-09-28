// TODO: maybe break apart into submodules, one for each subgroup
use toml;
use std::io::prelude::*;
use std::path::PathBuf;
use std::fs::OpenOptions;
use std::env;
use std::result;
use std::fmt;

pub type Result<T> = result::Result<T, String>;

#[derive(RustcDecodable)]
pub struct Configuration {
    pub client: ClientConfiguration,
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

        Ok(Configuration {
            client: client,
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

#[derive(RustcDecodable)]
pub struct ClientConfiguration {
    pub storage_dir: String,
    pub rvi_url: Option<String>,
    pub edge_url: Option<String>
}

impl ClientConfiguration {
    pub fn parse(tree: &toml::Table) -> Result<ClientConfiguration> {
        let client_tree = try!(tree.get("client")
            .ok_or("Missing required subgroup \"client\""));

        let storage_dir = try!(client_tree.lookup("storage_dir")
                               .ok_or("Missing required key \"storage_dir\""));
        let storage_dir = try!(value_to_string(storage_dir, "storage_dir"));

        let rvi_url = match client_tree.lookup("rvi_url") {
            Some(val) => { Some(try!(value_to_string(val, "rvi_url"))) },
            None => None
        };

        let edge_url = match client_tree.lookup("edge_url") {
            Some(val) => { Some(try!(value_to_string(val, "edge_url"))) },
            None => None
        };

        Ok(ClientConfiguration {
            storage_dir: storage_dir,
            rvi_url: rvi_url,
            edge_url: edge_url
        })
    }
}

fn value_to_string(val: &toml::Value, key: &str) -> Result<String> {
    val.as_str().map(|s| s.to_string())
        .ok_or(format!("Key \"{}\" is not a string", key))
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

    #[test]
    fn it_correctly_parses_a_valid_configuration() {
        test_init!();
        let data = format!(r#"
        [client]
        storage_dir = "{}"
        rvi_url = "{}"
        edge_url = "{}"
        "#, STORAGE, RVI, EDGE);

        let configuration = Configuration::parse(&data).unwrap();
        assert_eq!(&configuration.client.storage_dir, STORAGE);
        assert_eq!(&configuration.client.rvi_url.unwrap(), RVI);
        assert_eq!(&configuration.client.edge_url.unwrap(), EDGE);
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
        "#, STORAGE, RVI, EDGE);

        let configuration = Configuration::parse(&data).unwrap();
        assert_eq!(&configuration.client.storage_dir, STORAGE);
        assert_eq!(&configuration.client.rvi_url.unwrap(), RVI);
        assert_eq!(&configuration.client.edge_url.unwrap(), EDGE);
    }

    #[test]
    fn it_ignores_extra_groups() {
        test_init!();
        let data = format!(r#"
        [client]
        storage_dir = "{}"
        rvi_url = "{}"
        edge_url = "{}"

        [testgroup]
        is_empty = true
        "#, STORAGE, RVI, EDGE);

        let configuration = Configuration::parse(&data).unwrap();
        assert_eq!(&configuration.client.storage_dir, STORAGE);
        assert_eq!(&configuration.client.rvi_url.unwrap(), RVI);
        assert_eq!(&configuration.client.edge_url.unwrap(), EDGE);
    }

    #[test]
    fn it_requires_the_client_group() {
        test_init!();
        let data = "[testgroup]\nis_empty = true".to_string();
        match Configuration::parse(&data) {
            Ok(..) => panic!("Accepted invalid configuration!"),
            Err(e) => {
                assert_eq!(e, "Missing required subgroup \"client\""
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
        "#, RVI, EDGE);

        match Configuration::parse(&data) {
            Ok(..) => panic!("Accepted invalid configuration!"),
            Err(e) => {
                assert_eq!(e, "Missing required key \"storage_dir\""
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
        "#, STORAGE, EDGE);

        let configuration = Configuration::parse(&data).unwrap();
        assert_eq!(&configuration.client.storage_dir, STORAGE);
        assert_eq!(configuration.client.rvi_url, None);
        assert_eq!(&configuration.client.edge_url.unwrap(), EDGE);
    }

    #[test]
    fn it_doesnt_require_the_edge_url_key() {
        test_init!();
        let data = format!(r#"
        [client]
        storage_dir = "{}"
        rvi_url = "{}"
        "#, STORAGE, RVI);

        let configuration = Configuration::parse(&data).unwrap();
        assert_eq!(&configuration.client.storage_dir, STORAGE);
        assert_eq!(&configuration.client.rvi_url.unwrap(), RVI);
        assert_eq!(configuration.client.edge_url, None);
    }

    #[test]
    fn it_uses_xdg_config_home_if_available() {
        test_init!();
        env::remove_var("XDG_CONFIG_HOME");
        env::set_var("XDG_CONFIG_HOME", "/some/thing");
        assert_eq!(Configuration::default_path(),
                   "/some/thing/sota/client.toml".to_string());
    }

    #[test]
    fn it_falls_back_to_home_if_possible() {
        test_init!();
        env::remove_var("XDG_CONFIG_HOME");
        env::remove_var("HOME");
        env::set_var("HOME", "/some/thing");
        assert_eq!(Configuration::default_path(),
                   "/some/thing/.sota/client.toml".to_string());
    }

    #[test]
    fn it_falls_back_to_pwd() {
        test_init!();
        env::remove_var("XDG_CONFIG_HOME");
        env::remove_var("HOME");
        assert_eq!(Configuration::default_path(),
                   ".sota/client.toml".to_string());
    }
}
