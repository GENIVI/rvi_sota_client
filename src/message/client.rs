use std::vec::Vec;
use dbus::{Message, MessageItem, FromMessageItem, Error};
use super::package_id::PackageId;
use super::server::BackendServices;

pub enum Notification {
    Notify(UserMessage),
    Initiate(Vec<PackageId>),
    InstallReport(PackageReport),
    Report(Vec<PackageId>),
    Finish(PackageId),
    RequestReport
}

#[derive(RustcDecodable, Clone, PartialEq, Eq, Debug)]
pub struct UserPackage {
    pub package: PackageId,
    pub size: u64
}

impl From<UserPackage> for MessageItem {
    fn from(p: UserPackage) -> MessageItem {
        let package = MessageItem::from(&p.package);
        let size = MessageItem::from(p.size);
        MessageItem::Struct(vec!(package, size))
    }
}

pub struct UserMessage {
    pub packages: Vec<UserPackage>,
    pub services: BackendServices
}

#[derive(RustcEncodable, Debug, PartialEq, Eq)]
pub struct PackageReport {
    pub package: PackageId,
    pub status: bool,
    pub description: String
}

impl<'a> FromMessageItem<'a> for PackageReport {
    fn from(i: &'a MessageItem) -> Result<Self, ()> {
        let mut message = try!(match i {
            &MessageItem::Struct(ref val) => Ok(val.clone()),
            _ => Err(())
        });

        let description_item = try!(message.pop().ok_or(()));
        let description: &String =
            try!(FromMessageItem::from(&description_item));

        let status_item = try!(message.pop().ok_or(()));
        let status: bool = try!(FromMessageItem::from(&status_item));

        let package_item = try!(message.pop().ok_or(()));
        let package: PackageId = try!(FromMessageItem::from(&package_item));

        Ok(PackageReport {
            package: package,
            status: status,
            description: description.clone()
        })
    }
}

pub trait ParsePackageReport {
    fn parse(&self, package: PackageId) -> PackageReport;
}

impl<T, E> ParsePackageReport for Result<T, E>
    where T: ParsePackageReport, E: ParsePackageReport {
    fn parse(&self, package: PackageId) -> PackageReport {
        match self {
            &Ok(ref val) => val.parse(package),
            &Err(ref e) => e.parse(package)
        }
    }
}

impl ParsePackageReport for Message {
    fn parse(&self, package: PackageId) -> PackageReport {
        let argument = match self.get_items().pop() {
            Some(val) => val,
            None => {
                error!("Missing argument to installation report call");
                return PackageReport {
                    package: package,
                    status: false,
                    description: "Missing argument to dbus call".to_string()
                }
            }
        };

        let parse_result: Result<PackageReport, ()> =
            FromMessageItem::from(&argument);

        match parse_result {
            Ok(p) => p,
            Err(..) => {
                error!("Couldn't parse dbus message: {:?}", self);
                PackageReport {
                    package: package,
                    status: false,
                    description: "D-Bus parse error".to_string()
                }
            }
        }
    }
}

impl ParsePackageReport for Error {
    fn parse(&self, package: PackageId) -> PackageReport {
        let message = self.message()
            .unwrap_or(self.name()
            .unwrap_or("Unknown error"))
            .to_string();

        error!("Did not receive Package Installation report: {}: {}",
               self.name().unwrap_or("None"),
               self.message().unwrap_or("Unknown error"));

        PackageReport {
            package: package,
            status: false,
            description: message
        }
    }
}

#[cfg(test)]
mod test {
    use dbus::*;

    use super::*;
    use configuration::*;
    use test_library::generate_random_package;

    impl<'a> From<&'a PackageReport> for MessageItem {
        fn from(p: &PackageReport) -> MessageItem {
            let d: &str = &p.description;
            MessageItem::Struct(vec!(
                    MessageItem::from(&p.package),
                    MessageItem::from(p.status),
                    MessageItem::from(d)))
        }
    }

    #[test]
    fn it_properly_decodes_a_successful_packge_report_from_dbus() {
        for i in 1..20 {
            let report = PackageReport {
                package: generate_random_package(i),
                status: true,
                description: "Successfully installed package".to_string()
            };

            let message_item = MessageItem::from(&report);
            let decoded: PackageReport =
                FromMessageItem::from(&message_item).unwrap();

            assert_eq!(decoded, report);
        }
    }

    #[test]
    fn it_properly_decodes_a_failed_packge_report_from_dbus() {
        for i in 1..20 {
            let report = PackageReport {
                package: generate_random_package(i),
                status: false,
                description: "Some error".to_string()
            };

            let message_item = MessageItem::from(&report);
            let decoded: PackageReport =
                FromMessageItem::from(&message_item).unwrap();

            assert_eq!(decoded, report);
        }
    }

    #[test]
    fn it_decodes_a_valid_dbus_message_to_package_report() {
        for i in 1..20 {
            let package = generate_random_package(i);
            let report = PackageReport {
                package: package.clone(),
                status: true,
                description: "Successfully installed package".to_string()
            };

            let config = DBusConfiguration::gen_test();
            let mut message =
                Message::new_method_call(&config.name, "/", &config.interface,
                                         "FinishDownload").unwrap();
            let args = [MessageItem::from(&report)];
            message.append_items(&args);

            assert_eq!(message.parse(package), report);
        }
    }

    #[test]
    fn it_generates_a_failed_package_report_for_invalid_messages() {
        for i in 1..20 {
            let package = generate_random_package(i);
            let report = PackageReport {
                package: package.clone(),
                status: false,
                description: "Missing argument to dbus call".to_string()
            };

            let config = DBusConfiguration::gen_test();
            let message =
                Message::new_method_call(&config.name, "/", &config.interface,
                                         "FinishDownload").unwrap();

            assert_eq!(message.parse(package), report);
        }
    }

    #[test]
    fn it_generates_a_failed_package_report_from_error_messages() {
        for i in 1..20 {
            let package= generate_random_package(i);

            let error = Error::new_custom("TestError",
                                          "This is a generated error");
            let report = PackageReport {
                package: package.clone(),
                status: false,
                description: format!("{}", error.message().unwrap())
            };

            assert_eq!(error.parse(package), report);
        }
    }
}
