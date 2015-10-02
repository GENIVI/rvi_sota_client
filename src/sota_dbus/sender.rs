use std::sync::mpsc;
use std::convert::From;
use std::borrow::Cow;

use dbus::{Connection, BusType, MessageItem, Message, FromMessageItem};

use configuration::DBusConfiguration;
use message::{UserPackage, PackageId, PackageReport, Notification};
use message::ParsePackageReport;

pub enum Request {
    Notify(Vec<UserPackage>),
    Complete(PackageId),
    Report
}

pub struct Sender {
    config: DBusConfiguration,
    receiver: mpsc::Receiver<Request>,
    sender: mpsc::Sender<Notification>
}

impl Sender {
    pub fn new(c: DBusConfiguration,
               r: mpsc::Receiver<Request>,
               s: mpsc::Sender<Notification>) -> Sender {
        Sender {
            config: c,
            receiver: r,
            sender: s
        }
    }

    pub fn start(&self) {
        loop {
            match self.receiver.recv().unwrap() {
                Request::Notify(packages) => {
                    self.send_notify(packages);
                },
                Request::Complete(package) => {
                    let result = self.send_complete(package);
                    let report = Notification::InstallReport(result);
                    let _ = self.sender.send(report);
                },
                Request::Report => {
                    let result = self.request_report();
                    let report = Notification::Report(result);
                    let _ = self.sender.send(report);
                }
            }
        }
    }

    fn send_notify(&self, packages: Vec<UserPackage>) {
        let connection = Connection::get_private(BusType::Session).unwrap();
        let mut message =
            Message::new_method_call(&self.config.software_manager, "/",
                                     &self.config.software_manager, "Notify")
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

    fn send_complete(&self, package: PackageId) -> PackageReport {
        let connection = Connection::get_private(BusType::Session).unwrap();
        let mut message =
            Message::new_method_call(&self.config.software_manager, "/",
                                     &self.config.software_manager,
                                     "DownloadComplete").unwrap();

        let args = [MessageItem::from(&package)];
        message.append_items(&args);

        connection
            .send_with_reply_and_block(message, self.config.timeout)
            .parse(package)
    }

    fn request_report(&self) -> Vec<PackageId> {
        let connection = Connection::get_private(BusType::Session).unwrap();
        let message =
            Message::new_method_call(&self.config.software_manager, "/",
                                     &self.config.software_manager,
                                     "GetAllPackages").unwrap();

        match connection.send_with_reply_and_block(message,
                                                   self.config.timeout) {
            Ok(ref val) => parse_package_list(val),
            Err(..) => Vec::new()
        }
    }
}

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
    use std::sync::mpsc::channel;
    use dbus::{Message, MessageItem};

    use super::*;
    use super::parse_package_list;

    use configuration::DBusConfiguration;
    use message::UserPackage;
    use test_library::generate_random_package;

    #[test]
    fn it_sets_a_valid_notify_signature() {
        test_init!();
        let (tx, _) = channel();
        let (_, rx) = channel();
        let sender = Sender::new(DBusConfiguration::gen_test(), rx, tx);
        let packages = vec!(UserPackage {
            package: generate_random_package(15),
            size: 500
        });

        sender.send_notify(packages);
    }

    #[test]
    fn it_sets_a_valid_download_complete_signature() {
        test_init!();
        let (tx, _) = channel();
        let (_, rx) = channel();
        let sender = Sender::new(DBusConfiguration::gen_test(), rx, tx);

        let _ = sender.send_complete(generate_random_package(15));
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
