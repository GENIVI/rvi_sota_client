//! Handles the `dbus` section of the configuration file.

use toml;

use super::common::{get_required_key, get_optional_key, ConfTreeParser, Result};

/// Type to encode allowed keys for the `dbus` section of the configuration.
#[derive(Clone)]
pub struct DBusConfiguration {
    /// The DBus name sota_client registers.
    pub name: String,
    /// The interface name sota_client provides.
    pub interface: String,
    /// The name and interface, where the software loading manager can be reached.
    pub software_manager: String,
    /// Time to wait for installation of a package before it is considered a failure. In seconds.
    pub timeout: i32 // dbus-rs expects a signed int
}

#[cfg(test)]
impl DBusConfiguration {
    /// Generate a test configuration.
    pub fn gen_test() -> DBusConfiguration {
        DBusConfiguration {
            name: "org.test.test".to_string(),
            interface: "org.test.test".to_string(),
            software_manager: "org.test.software_manager".to_string(),
            timeout: 20
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
            timeout: timeout.unwrap_or(60) * 1000
        })
    }
}

#[cfg(test)] static NAME: &'static str = "org.genivi.sota_client";
#[cfg(test)] static INTERFACE: &'static str = "org.genivi.software_manager";
#[cfg(test)] static SOFTWARE_MANAGER: &'static str = "org.genivi.software_manager";

#[cfg(test)]
pub fn gen_valid_conf() -> String {
    format!(r#"
    [dbus]
    name = "{}"
    interface = "{}"
    software_manager = "{}"
    "#, NAME, INTERFACE, SOFTWARE_MANAGER)
}

#[cfg(test)]
pub fn assert_conf(conf: &DBusConfiguration) -> bool {
    assert_eq!(&conf.name, NAME);
    assert_eq!(&conf.interface, INTERFACE);
    assert_eq!(&conf.software_manager, SOFTWARE_MANAGER);
    true
}

#[cfg(test)]
mod test {
    use super::*;
    use super::{NAME, INTERFACE, SOFTWARE_MANAGER};
    use configuration::common::{ConfTreeParser, read_tree};

    #[test]
    fn it_requires_the_dbus_name_key() {
        test_init!();
        let data = format!(r#"
        [dbus]
        interface = "{}"
        software_manager = "{}"
        "#, INTERFACE, SOFTWARE_MANAGER);

        let tree = read_tree(&data).unwrap();
        match DBusConfiguration::parse(&tree) {
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
        [dbus]
        name = "{}"
        software_manager = "{}"
        "#, NAME, SOFTWARE_MANAGER);

        let tree = read_tree(&data).unwrap();
        match DBusConfiguration::parse(&tree) {
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
        [dbus]
        name = "{}"
        interface = "{}"
        "#, NAME, INTERFACE);

        let tree = read_tree(&data).unwrap();
        match DBusConfiguration::parse(&tree) {
            Ok(..) => panic!("Accepted invalid configuration!"),
            Err(e) => {
                assert_eq!(e, "Missing required key \"software_manager\" \
                           in \"dbus\"".to_string());
            }
        };
    }
}
