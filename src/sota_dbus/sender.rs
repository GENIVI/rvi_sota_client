//! Sending side of the DBus interface.

use std::convert::From;
use std::borrow::Cow;

use dbus::{Connection, BusType, MessageItem, Message, FromMessageItem};

use configuration::DBusConfiguration;
use message::{UserPackage, PackageId, PackageReport};
use message::ParsePackageReport;

/// Foward a "Notify" message to DBus.
///
/// # Arguments
/// * `config`: The configuration of the DBus interface.
/// * `packages`: `Vector` of the packages that need updating.
pub fn send_notify(config: &DBusConfiguration, packages: Vec<UserPackage>) {
    let connection = Connection::get_private(BusType::Session).unwrap();
    let mut message =
        Message::new_method_call(&config.software_manager, "/",
                                 &config.software_manager, "Notify")
        .unwrap();

    let mut message_items = Vec::new();
    for package in packages {
        message_items.push(MessageItem::from(package));
    }

    // hardcoded signature as a workaround for diwic/dbus-rs#24
    // needs to stay in until the fix is released and works on stable
    let args = [MessageItem::Array(message_items,
                                   Cow::Owned("(a{ss}t)".to_string()))];

    message.append_items(&args);
    if connection.send(message).is_err() {
        error!("Couldn't forward message to D-Bus");
    }
}

/// Ask the Software Loading Manager to isntall a package. Will block until the installation
/// finished or the timeout is reached.
///
/// # Arguments
/// * `config`: The configuration of the DBus interface.
/// * `package`: The package to install.
pub fn request_install(config: &DBusConfiguration, package: PackageId)
    -> PackageReport {
        let connection = Connection::get_private(BusType::Session).unwrap();
        let mut message =
            Message::new_method_call(&config.software_manager, "/",
                                     &config.software_manager,
                                     "DownloadComplete").unwrap();

        let args = [MessageItem::from(&package)];
        message.append_items(&args);

        connection
            .send_with_reply_and_block(message, config.timeout)
            .parse(package)
    }

/// Request a full report from the Software Loading Manager. Will block until the list of all
/// installed packages is received or the timeout is reached.
///
/// # Arguments
/// * `config`: The configuration of the DBus interface.
pub fn request_report(config: &DBusConfiguration) -> Vec<PackageId> {
    let connection = Connection::get_private(BusType::Session).unwrap();
    let message =
        Message::new_method_call(&config.software_manager, "/",
                                 &config.software_manager,
                                 "GetAllPackages").unwrap();

    match connection.send_with_reply_and_block(message,
                                               config.timeout) {
        Ok(ref val) => parse_package_list(val),
        Err(..) => Vec::new()
    }
}

/// Parses an incoming DBus message to a `Vector` of `PackageId`s. Ignores unparsable entries, thus
/// an empty Vector might indicate a parser error.
///
/// # Arguments
/// * `m`: The message to parse.
fn parse_package_list(m: &Message) -> Vec<PackageId> {
    let argument = match m.get_items().pop() {
        Some(val) => val,
        None => return Vec::new()
    };

    let package_items = match argument {
        MessageItem::Array(val, _) => val,
        _ => return Vec::new()
    };

    let mut packages = Vec::new();
    for item in package_items {
        let package: PackageId = match FromMessageItem::from(&item) {
            Ok(val) => val,
            Err(..) => continue
        };
        packages.push(package);
    }

    packages
}

#[cfg(test)]
mod test {
    use dbus::{Message, MessageItem};

    use super::*;
    use super::parse_package_list;

    use configuration::DBusConfiguration;
    use message::UserPackage;
    use test_library::generate_random_package;

    #[test]
    fn it_sets_a_valid_notify_signature() {
        test_init!();
        let conf = DBusConfiguration::gen_test();
        let packages = vec!(UserPackage {
            package: generate_random_package(15),
            size: 500
        });

        send_notify(&conf, packages);
    }

    #[test]
    fn it_sets_a_valid_download_complete_signature() {
        test_init!();
        let conf = DBusConfiguration::gen_test();
        request_install(&conf, generate_random_package(15));
    }

    fn gen_test_message() -> Message {
        let config = DBusConfiguration::gen_test();
        Message::new_method_call(&config.name, "/", &config.interface,
                                 "GetAllPackages").unwrap()
    }

    #[test]
    fn it_successfully_parses_a_valid_report() {
        test_init!();
        let mut message = gen_test_message();
        let mut packages = Vec::new();
        let mut package_items = Vec::new();
        for i in 1..20 {
            let package = generate_random_package(i);
            package_items.push(MessageItem::from(&package));
            packages.push(package);
        }

        let args = [MessageItem::new_array(package_items).unwrap()];
        message.append_items(&args);

        let decoded = parse_package_list(&message);
        assert!(!decoded.is_empty());
        assert_eq!(decoded, packages);
    }

    #[test]
    fn it_returns_a_empty_list_for_invalid_reports() {
        test_init!();
        let message = gen_test_message();
        let decoded = parse_package_list(&message);
        assert!(decoded.is_empty());
    }
}
