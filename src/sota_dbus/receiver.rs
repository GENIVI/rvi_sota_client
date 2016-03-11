//! Receiving side of the DBus interface.

use std::sync::mpsc::Sender;

use configuration::DBusConfiguration;
use event::Event;
use event::outbound::{OutBoundEvent, OperationResults, UpdateReport};

use dbus::{Connection, NameFlag, BusType, ConnectionItem, Message, FromMessageItem};
use dbus::obj::*;

/// DBus error string to indicate a missing argument.
static MISSING_ARG: &'static str = "Error.MissingArgument";
/// DBus error string to indicate a malformed argument.
static MALFORMED_ARG: &'static str = "Error.MalformedArgument";

/// Format a DBus error message indicating a missing argument.
fn missing_arg() -> (&'static str, String) {
    (MISSING_ARG, "Missing argument".to_string())
}

/// Format a DBus error message indicating a malformed argument.
fn malformed_arg() -> (&'static str, String) {
    (MALFORMED_ARG, "Malformed argument".to_string())
}

/// Encodes the state that is needed to accept incoming DBus messages.
pub struct Receiver {
    /// The configuration for the DBus interface.
    config: DBusConfiguration,
    /// A sender to forward incoming messages.
    sender: Sender<Event>
}

impl Receiver {
    /// Create a new `Receiver`.
    ///
    /// # Arguments
    /// * `c`: The configuration for the DBus interface.
    /// * `s`: A sender to forward incoming messages.
    pub fn new(c: DBusConfiguration, s: Sender<Event>) -> Receiver {
        Receiver {
            config: c,
            sender: s
        }
    }

    /// Start the listener. It will register in DBus according to the configuration, wait for
    /// incoming messages and forward them via the internal `Sender`.
    pub fn start(&self) {
        let conn = Connection::get_private(BusType::Session).unwrap();
        conn.register_name(&self.config.name, NameFlag::ReplaceExisting as u32).unwrap();

        let initiate_download = Method::new(
            "initiate_method",
            vec!(Argument::new("update_id", "s")),
            vec!(),
            Box::new(|msg| self.handle_initiate_download(msg)));
        let abort_download = Method::new(
            "abort_download",
            vec!(Argument::new("update_id", "s")),
            vec!(),
            Box::new(|msg| self.handle_abort_download(msg)));
        let update_report = Method::new(
            "update_report",
            vec!(Argument::new("update_id", "s"), Argument::new("operations_results", "a(a{sis})")),
            vec!(),
            Box::new(|msg| self.handle_update_report(msg)));
        let interface = Interface::new(vec!(initiate_download, abort_download, update_report), vec!(), vec!());

        let mut object_path = ObjectPath::new(&conn, "/", true);
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

    /// Handles incoming "Initiate Download" messages.
    ///
    /// Parses the message and forwards it to the internal `Sender`.
    ///
    /// # Arguments
    /// * `msg`: The message to handle.
    fn handle_initiate_download(&self, msg: &mut Message) -> MethodResult {
        let sender = try!(get_sender(msg).ok_or(missing_arg()));
        trace!("sender: {:?}", sender);
        trace!("msg: {:?}", msg);

        let arg = try!(msg.get_items().pop().ok_or(missing_arg()));
        let update_id: &String = try!(FromMessageItem::from(&arg).or(Err(malformed_arg())));
        let _ = self.sender.send(
            Event::OutBound(OutBoundEvent::InitiateDownload(update_id.clone())));

        Ok(vec!())
    }

    fn handle_abort_download(&self, msg: &mut Message) -> MethodResult {
        let sender = try!(get_sender(msg).ok_or(missing_arg()));
        trace!("sender: {:?}", sender);
        trace!("msg: {:?}", msg);

        let arg = try!(msg.get_items().pop().ok_or(missing_arg()));
        let update_id: &String = try!(FromMessageItem::from(&arg).or(Err(malformed_arg())));
        let _ = self.sender.send(
            Event::OutBound(OutBoundEvent::AbortDownload(update_id.clone())));

        Ok(vec!())
    }

    fn handle_update_report(&self, msg: &mut Message) -> MethodResult {
        let sender = try!(get_sender(msg).ok_or(missing_arg()));
        trace!("sender: {:?}", sender);
        trace!("msg: {:?}", msg);

        let arg = try!(msg.get_items().pop().ok_or(missing_arg()));
        let update_id: &String = try!(FromMessageItem::from(&arg).or(Err(malformed_arg())));

        let arg = try!(msg.get_items().pop().ok_or(missing_arg()));
        let operation_results: OperationResults = try!(FromMessageItem::from(&arg).or(Err(malformed_arg())));

        let report = UpdateReport::new(update_id.clone(), operation_results);
        let _ = self.sender.send(
            Event::OutBound(OutBoundEvent::UpdateReport(report)));

        Ok(vec!())
    }
}

#[cfg(not(test))]
fn get_sender(msg: &Message) -> Option<String> { msg.sender() }
#[cfg(test)]
fn get_sender(_: &Message) -> Option<String> { Some("test".to_string()) }


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
        receiver.handle_initiate_download(&mut message).unwrap();

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
        receiver.handle_initiate_download(&mut message).unwrap_err();

        match rx.try_recv() {
            Err(TryRecvError::Empty) => {},
            Err(TryRecvError::Disconnected) => panic!("Closed channel!"),
            Ok(..) => panic!("Forwarded invalid message!")
        }
    }
}
