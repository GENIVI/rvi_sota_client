use toml;

use super::common::{get_required_key, get_optional_key, ConfTreeParser, Result};

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

#[cfg(test)] static STORAGE: &'static str = "/var/sota";
#[cfg(test)] static RVI: &'static str = "/http://localhost:8901";
#[cfg(test)] static EDGE: &'static str = "localhost:9080";

#[cfg(test)]
pub fn gen_valid_conf() -> String {
    format!(r#"
    [client]
    storage_dir = "{}"
    rvi_url = "{}"
    edge_url = "{}"
    "#, STORAGE, RVI, EDGE)
}

#[cfg(test)]
pub fn assert_conf(conf: &ClientConfiguration) -> bool {
    assert_eq!(&conf.storage_dir, STORAGE);
    assert_eq!(&conf.rvi_url.clone().unwrap(), RVI);
    assert_eq!(&conf.edge_url.clone().unwrap(), EDGE);
    true
}

#[cfg(test)]
pub mod test {
    use super::*;
    use super::{STORAGE, RVI, EDGE};
    use configuration::common::{ConfTreeParser, read_tree};

    #[test]
    fn it_requires_the_storage_dir_key() {
        test_init!();
        let data = format!(r#"
        [client]
        rvi_url = "{}"
        edge_url = "{}"
        "#, RVI, EDGE);

        let tree = read_tree(&data).unwrap();
        match ClientConfiguration::parse(&tree) {
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
        "#, STORAGE, EDGE);

        let tree = read_tree(&data).unwrap();
        let configuration = ClientConfiguration::parse(&tree).unwrap();
        assert_eq!(&configuration.storage_dir, STORAGE);
        assert_eq!(configuration.rvi_url, None);
        assert_eq!(&configuration.edge_url.unwrap(), EDGE);
    }

    #[test]
    fn it_doesnt_require_the_edge_url_key() {
        test_init!();
        let data = format!(r#"
        [client]
        storage_dir = "{}"
        rvi_url = "{}"
        "#, STORAGE, RVI);

        let tree = read_tree(&data).unwrap();
        let configuration = ClientConfiguration::parse(&tree).unwrap();
        assert_eq!(&configuration.storage_dir, STORAGE);
        assert_eq!(&configuration.rvi_url.unwrap(), RVI);
        assert_eq!(configuration.edge_url, None);
    }
}
