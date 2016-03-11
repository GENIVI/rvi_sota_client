//! Sending side of the DBus interface.

use std::convert::From;

use dbus::{Connection, BusType, MessageItem, Message, FromMessageItem};

use configuration::DBusConfiguration;
use event::inbound::{UpdateAvailable, DownloadComplete, GetInstalledSoftware};
use event::outbound::{InstalledFirmwares, InstalledPackages, InstalledSoftware};

// use message::{UserPackage, PackageId, PackageReport};
// use message::ParsePackageReport;

pub fn send_update_available(config: &DBusConfiguration, e: UpdateAvailable) {
    let args = [
        MessageItem::from(e.update_id),
        MessageItem::from(e.signature),
        MessageItem::from(e.description),
        MessageItem::from(e.request_confirmation)];
    let mut message = Message::new_method_call(
        &config.software_manager, "/",
        &config.software_manager, "update_available").unwrap();
    message.append_items(&args);

    let conn = Connection::get_private(BusType::Session).unwrap();
    let _ = conn.send(message)
        .map_err(|_| error!("Couldn't forward message to D-Bus"));
}

pub fn send_download_complete(config: &DBusConfiguration, e: DownloadComplete) {
    let args = [
        MessageItem::from(e.update_image),
        MessageItem::from(e.signature)];
    let mut message = Message::new_method_call(
        &config.software_manager, "/",
        &config.software_manager, "download_complete").unwrap();
    message.append_items(&args);

    let conn = Connection::get_private(BusType::Session).unwrap();
    let _ = conn.send(message)
        .map_err(|_| error!("Couldn't forward message to D-Bus"));
}

pub fn send_get_installed_software(config: &DBusConfiguration, e: GetInstalledSoftware)
    -> Result<InstalledSoftware, ()> {
    let args = [
        MessageItem::from(e.include_packages),
        MessageItem::from(e.include_module_firmware)];
    let mut message = Message::new_method_call(
        &config.software_manager, "/",
        &config.software_manager, "get_installed_software").unwrap();
    message.append_items(&args);

    let conn = Connection::get_private(BusType::Session).unwrap();
    let msg = conn.send_with_reply_and_block(message, config.timeout).unwrap();

    let arg = try!(msg.get_items().pop().ok_or(()));
    let installed_packages: InstalledPackages = try!(FromMessageItem::from(&arg));

    let arg = try!(msg.get_items().pop().ok_or(()));
    let installed_firmware: InstalledFirmwares = try!(FromMessageItem::from(&arg));

    Ok(InstalledSoftware {
        packages: installed_packages,
        firmware: installed_firmware
    })
}

/*
 *
use std::borrow::Cow;

/// Foward a "Notify" message to DBus.
///
/// # Arguments
/// * `config`: The configuration of the DBus interface.
/// * `packages`: `Vector` of the packages that need updating.
pub fn send_notify(config: &DBusConfiguration, packages: Vec<UserPackage>) {
    let items = packages.iter().map(|p| MessageItem::from(p.clone())).collect();
    // hardcoded signature as a workaround for diwic/dbus-rs#24
    // needs to stay in until the fix is released and works on stable
    let args = [MessageItem::Array(items, Cow::Owned("(a{ss}t)".to_string()))];

    let mut message = Message::new_method_call(
        &config.software_manager, "/",
        &config.software_manager, "Notify").unwrap();
    message.append_items(&args);

    let conn = Connection::get_private(BusType::Session).unwrap();
    let _ = conn.send(message)
        .map_err(|_| error!("Couldn't forward message to D-Bus"));
}

/// Ask the Software Loading Manager to isntall a package. Will block until the installation
/// finished or the timeout is reached.
///
/// # Arguments
/// * `config`: The configuration of the DBus interface.
/// * `package`: The package to install.
pub fn request_install(config: &DBusConfiguration, package: PackageId) -> PackageReport {
    let args = [MessageItem::from(&package)];
    let mut message = Message::new_method_call(
        &config.software_manager, "/",
        &config.software_manager, "DownloadComplete").unwrap();
    message.append_items(&args);

    let conn = Connection::get_private(BusType::Session).unwrap();
    conn.send_with_reply_and_block(message, config.timeout)
        .parse(package)
}

/// Request a full report from the Software Loading Manager. Will block until the list of all
/// installed packages is received or the timeout is reached.
///
/// # Arguments
/// * `config`: The configuration of the DBus interface.
pub fn request_report(config: &DBusConfiguration) -> Vec<PackageId> {
    let message = Message::new_method_call(
        &config.software_manager, "/",
        &config.software_manager, "GetAllPackages").unwrap();

    let conn = Connection::get_private(BusType::Session).unwrap();
    conn.send_with_reply_and_block(message, config.timeout)
        .map(|v| parse_package_list(&v))
        .unwrap_or(Vec::new())
}

/// Parses an incoming DBus message to a `Vector` of `PackageId`s. Ignores unparsable entries, thus
/// an empty Vector might indicate a parser error.
///
/// # Arguments
/// * `m`: The message to parse.
fn parse_package_list(m: &Message) -> Vec<PackageId> {
    m.get_items().pop()
        .and_then(|item| match item { MessageItem::Array(val, _) => Some(val), _ => None})
        .map(|arr| arr.iter().map(|p| FromMessageItem::from(p).unwrap()).collect())
        .unwrap_or(Vec::new())
}
*/

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
