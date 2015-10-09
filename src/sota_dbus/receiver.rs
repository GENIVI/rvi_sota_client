use std::sync::mpsc::Sender;

use configuration::DBusConfiguration;
use message::{PackageId, Notification};

use dbus::{Connection, NameFlag, BusType, MessageItem, ConnectionItem, Message};
use dbus::FromMessageItem;
use dbus::obj::*;

static MISSING_ARG: &'static str = "Error.MissingArgument";
static MALFORMED_ARG: &'static str = "Error.MalformedArgument";

fn missing_arg() -> (&'static str, String) {
    (MISSING_ARG, "Missing argument".to_string())
}

fn malformed_arg() -> (&'static str, String) {
    (MALFORMED_ARG, "Malformed argument".to_string())
}

pub struct Receiver {
    config: DBusConfiguration,
    sender: Sender<Notification>
}

impl Receiver {
    pub fn new(c: DBusConfiguration, s: Sender<Notification>) -> Receiver {
        Receiver {
            config: c,
            sender: s
        }
    }

    pub fn start(&self) {
        let conn = Connection::get_private(BusType::Session).unwrap();
        conn.register_name(&self.config.name,
                           NameFlag::ReplaceExisting as u32).unwrap();
        let mut object_path = ObjectPath::new(&conn, "/", true);

        let initiate_method =
            Method::new("InitiateDownload",
                        vec!(Argument::new("PackageId", "a{ss}")),
                        vec!(Argument::new("Status", "b")),
                        Box::new(|msg| self.handle_initiate(msg)));

        let interface = Interface::new(vec!(initiate_method), vec!(), vec!());

        object_path.insert_interface(&self.config.interface, interface);
        object_path.set_registered(true).unwrap();

        for n in conn.iter(1000) {
            match n {
                ConnectionItem::MethodCall(mut m) => {
                    object_path.handle_message(&mut m);
                },
                _ => {}
            }
        }
    }

    fn handle_initiate(&self, msg: &mut Message) -> MethodResult {
        trace!("msg: {:?}", msg);
        let arg = try!(msg.get_items().pop().ok_or(missing_arg()));
        let sender = try!(get_sender(msg).ok_or(missing_arg()));
        trace!("sender: {:?}", sender);
        let packages = try!(parse_package_list(&arg, &sender)
                            .or(Err(malformed_arg())));

        let message = Notification::Initiate(packages);
        let _ = self.sender.send(message);

        Ok(vec!(MessageItem::Bool(true)))
    }
}

#[cfg(not(test))]
fn get_sender(msg: &Message) -> Option<String> { msg.sender() }
#[cfg(test)]
fn get_sender(_: &Message) -> Option<String> { Some("test".to_string()) }

fn parse_package_list(msg: &MessageItem, sender: &str)
    -> Result<PackageId, ()> {
    let package: PackageId = try!(FromMessageItem::from(msg));
    info!("Got initiate for {} from {}", package, sender);
    Ok(package)
}

#[cfg(test)]
mod test {
    use std::sync::mpsc::{channel, TryRecvError};
    use std::convert::From;
    use dbus::{Message, MessageItem};

    use super::*;
    use message::Notification;
    use configuration::DBusConfiguration;
    use test_library::generate_random_package;

    macro_rules! setup_receiver {
        () => {{
            let (tx, rx) = channel();
            let config = DBusConfiguration::gen_test();
            let receiver = Receiver::new(config.clone(), tx);
            let message =
                Message::new_method_call(&config.name, "/", &config.interface,
                                        "InitiateDownload").unwrap();
            (rx, receiver, message)
        }}
    }

    #[test]
    fn it_forwards_correct_initiate_messages() {
        test_init!();
        let (rx, receiver, mut message) = setup_receiver!();
        let package = generate_random_package(15);
        let args = [MessageItem::from(&package)];
        message.append_items(&args);
        receiver.handle_initiate(&mut message).unwrap();

        match rx.try_recv().unwrap() {
            Notification::Initiate(val) => {
                assert_eq!(val, package);
            },
            _ => panic!("Didn't receive initiate notification!")
        }
    }

    #[test]
    fn it_returns_an_error_on_incorrect_messages() {
        test_init!();
        let (rx, receiver, mut message) = setup_receiver!();
        let args = [MessageItem::Str("error".to_string())];
        message.append_items(&args);
        receiver.handle_initiate(&mut message).unwrap_err();

        match rx.try_recv() {
            Err(TryRecvError::Empty) => {},
            Err(TryRecvError::Disconnected) => panic!("Closed channel!"),
            Ok(..) => panic!("Forwarded invalid message!")
        }
    }
}
