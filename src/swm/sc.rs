//! Receiving side of the DBus interface.

use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use chan::Sender;

use dbus::{Connection, NameFlag, BusType, ConnectionItem, Message, FromMessageItem};
use dbus::obj::*;

use rustc_serialize::{Decodable, Encodable};

use datatype::config::DBusConfiguration;
use datatype::command::Command;
use datatype::report::{UpdateReport, OperationResults};
use interaction_library::gateway::{Gateway, Interpret};

use super::dbus::*;


/// Encodes the state that is needed to accept incoming DBus messages.
pub struct SotaC<C, E>
    where C: Decodable + Send + Clone + Debug + 'static,
          E: Encodable + Send + Clone + Debug + 'static {
    /// The configuration for the DBus interface.
    config: DBusConfiguration,
    /// A sender to forward incoming messages.
    sender: Arc<Mutex<Sender<Interpret<C, E>>>>,
}

impl<E> SotaC<Command, E>
    where E: Encodable + Send + Clone + Debug + 'static {
    /// Create a new `SotaC`.
    ///
    /// # Arguments
    /// * `c`: The configuration for the DBus interface.
    /// * `s`: A sender to forward incoming messages.
    pub fn new(c: DBusConfiguration, tx: Sender<Interpret<Command, E>>) -> SotaC<Command, E> {
        SotaC {
            config: c,
            sender: Arc::new(Mutex::new(tx)),
        }
    }

    /// Start the listener. It will register in DBus according to the configuration, wait for
    /// incoming messages and forward them via the internal `Sender`.
    pub fn start(&self) {
        let conn = Connection::get_private(BusType::Session).unwrap();
        conn.register_name(&self.config.name, NameFlag::ReplaceExisting as u32).unwrap();

        let initiate_download = Method::new(
            "initiateDownload",
            vec!(Argument::new("update_id", "s")),
            vec!(),
            Box::new(|msg| self.handle_initiate_download(msg)));
        let abort_download = Method::new(
            "abortDownload",
            vec!(Argument::new("update_id", "s")),
            vec!(),
            Box::new(|msg| self.handle_abort_download(msg)));
        let update_report = Method::new(
            "updateReport",
            vec!(Argument::new("update_id", "s"), Argument::new("operations_results", "aa{sv}")),
            vec!(),
            Box::new(|msg| self.handle_update_report(msg)));
        let interface = Interface::new(vec!(initiate_download, abort_download, update_report), vec!(), vec!());

        let mut object_path = ObjectPath::new(&conn, &self.config.path, true);
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

        let mut args = msg.get_items().into_iter();
        let arg = try!(args.next().ok_or(missing_arg()));
        let update_id: &String = try!(FromMessageItem::from(&arg).or(Err(malformed_arg())));
        self.send(Command::AcceptUpdates(vec![update_id.clone()]));

        Ok(vec!())
    }

    fn handle_abort_download(&self, msg: &mut Message) -> MethodResult {
        let sender = try!(get_sender(msg).ok_or(missing_arg()));
        trace!("sender: {:?}", sender);
        trace!("msg: {:?}", msg);

        // TODO: Implement feature
        /*
        let mut args = msg.get_items().into_iter();
        let arg = try!(args.next().ok_or(missing_arg()));
        let update_id: &String = try!(FromMessageItem::from(&arg).or(Err(malformed_arg())));
        self.send(Command::AbortDownload(update_id.clone()));
        */

        Ok(vec!())
    }

    fn handle_update_report(&self, msg: &mut Message) -> MethodResult {
        let sender = try!(get_sender(msg).ok_or(missing_arg()));
        trace!("sender: {:?}", sender);
        trace!("msg: {:?}", msg);

        let mut args = msg.get_items().into_iter();
        let arg = try!(args.next().ok_or(missing_arg()));
        let update_id: &String = try!(FromMessageItem::from(&arg).or(Err(malformed_arg())));

        let arg = try!(args.next().ok_or(missing_arg()));
        let operation_results: OperationResults = try!(FromMessageItem::from(&arg).or(Err(malformed_arg())));

        let report = UpdateReport::new(update_id.clone(), operation_results);
        self.send(Command::UpdateReport(report));

        Ok(vec!())
    }

    fn send(&self, c: Command) {
        let _ = self.sender.lock().unwrap().send(
            Interpret { command: c, response_tx: None });
    }
}

fn get_sender(msg: &Message) -> Option<String> {
    msg.sender().map(|s| s.to_string())
}



impl<E> Gateway<Command, E> for SotaC<Command, E>
    where E: Encodable + Send + Clone + Debug + 'static,
{
    fn new(tx: Sender<Interpret<Command, E>>) -> Result<Self, String> {
        Ok(SotaC::new(DBusConfiguration::default(), tx))
    }
}

