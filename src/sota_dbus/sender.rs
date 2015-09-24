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
        // TODO: SOTA-129
        Vec::new()
    }
}

#[cfg(test)]
mod test {
    use std::sync::mpsc::channel;

    use super::*;
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
}
