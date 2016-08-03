use chan::Sender;
use dbus::{Connection, BusType, ConnectionItem, FromMessageItem,
           Message, MessageItem, NameFlag};
use dbus::obj::{Argument, Interface, Method, MethodResult, ObjectPath};
use std::thread;
use std::convert::From;

use datatype::{Command, DBusConfig, Event, InstalledFirmware, InstalledPackage,
               InstalledSoftware, OperationResult, UpdateReport};
use datatype::dbus;
use super::{Gateway, Interpret};


pub struct DBus {
    pub dbus_cfg:  DBusConfig,
    pub itx:       Sender<Interpret>,
}

impl Gateway for DBus {
    fn initialize(&mut self, itx: Sender<Interpret>) -> Result<(), String> {
        let dbus_cfg = self.dbus_cfg.clone();

        thread::spawn(move || {
            let conn = Connection::get_private(BusType::Session).expect("couldn't get dbus session");
            conn.register_name(&dbus_cfg.name, NameFlag::ReplaceExisting as u32).unwrap();

            let mut obj_path = ObjectPath::new(&conn, &dbus_cfg.path, true);
            obj_path.insert_interface(&dbus_cfg.interface, default_interface(itx));
            obj_path.set_registered(true).expect("couldn't set registration status");

            loop {
                for item in conn.iter(1000) {
                    match item {
                        ConnectionItem::MethodCall(mut msg) => {
                            info!("dbus method call: {:?}", msg);
                            obj_path.handle_message(&mut msg).map(|result| {
                                let _ = result.map_err(|_| error!("dbus method call failed: {:?}", msg));
                            });
                        }
                        _ => ()
                    }
                }
            }
        });

        Ok(info!("DBus gateway started."))
    }

    fn pulse(&self, event: Event) {
        match event {
            Event::UpdateAvailable(avail) => {
                let msg = self.new_message("updateAvailable", &[
                    MessageItem::from(avail.update_id),
                    MessageItem::from(avail.signature),
                    MessageItem::from(avail.description),
                    MessageItem::from(avail.confirmation)
                ]);
                let conn = Connection::get_private(BusType::Session).expect("couldn't get dbus session");
                let _    = conn.send(msg).map_err(|_| error!("couldn't send updateAvailable message"));
            }

            Event::DownloadComplete(comp) => {
                let msg = self.new_message("downloadComplete", &[
                    MessageItem::from(comp.update_image),
                    MessageItem::from(comp.signature)
                ]);
                let conn = Connection::get_private(BusType::Session).expect("couldn't get dbus session");
                let _    = conn.send(msg).map_err(|_| error!("couldn't send downloadComplete message"));
            }

            Event::GetInstalledSoftware(get) => {
                let msg = self.new_message("getInstalledPackages", &[
                    MessageItem::from(get.include_packages),
                    MessageItem::from(get.include_firmware)
                ]);
                let conn  = Connection::get_private(BusType::Session).expect("couldn't get dbus session");
                let reply = conn.send_with_reply_and_block(msg, self.dbus_cfg.timeout).unwrap();

                let _ = || -> Result<InstalledSoftware, ()> {
                    let mut args = reply.get_items().into_iter();

                    let pkg_arg  = try!(args.next().ok_or(()));
                    let msgs: &Vec<MessageItem> = try!(FromMessageItem::from(&pkg_arg));
                    let packages = try!(msgs.into_iter()
                                        .map(|item| -> Result<InstalledPackage, ()> {
                                            FromMessageItem::from(item)
                                        }).collect::<Result<Vec<InstalledPackage>, ()>>());

                    let firm_arg = try!(args.next().ok_or(()));
                    let msgs: &Vec<MessageItem> = try!(FromMessageItem::from(&firm_arg));
                    let firmwares = try!(msgs.into_iter()
                                         .map(|item| -> Result<InstalledFirmware, ()> {
                                             FromMessageItem::from(item)
                                         }).collect::<Result<Vec<InstalledFirmware>, ()>>());

                    Ok(InstalledSoftware::new(packages, firmwares))
                }().map(|inst| send(&self.itx, Command::SendInstalledSoftware(Some(inst))))
                   .map_err(|_| error!("unable to ReportInstalledSoftware"));
            }

            _ => ()
        }
    }
}

impl DBus {
    fn new_message(&self, method: &str, args: &[MessageItem]) -> Message {
        let mgr     = self.dbus_cfg.software_manager.clone();
        let path    = self.dbus_cfg.software_manager_path.clone();
        let result  = Message::new_method_call(&mgr, &path, &mgr, method);
        let mut msg = result.expect("couldn't create dbus message");
        msg.append_items(&args);
        msg
    }
}

fn default_interface<'i>(itx: Sender<Interpret>) -> Interface<'i> {
    let initiate_itx      = itx.clone();
    let initiate_download = Method::new(
        "initiateDownload",
        vec![Argument::new("update_id", "s")],
        vec![],
        Box::new(move |msg| handle_initiate_download(&initiate_itx, msg))
    );

    let abort_itx      = itx.clone();
    let abort_download = Method::new(
        "abortDownload",
        vec![Argument::new("update_id", "s")],
        vec![],
        Box::new(move |msg| handle_abort_download(&abort_itx, msg))
    );

    let update_itx    = itx.clone();
    let update_report = Method::new(
        "updateReport",
        vec![Argument::new("update_id", "s"), Argument::new("operations_results", "aa{sv}")],
        vec![],
        Box::new(move |msg| handle_update_report(&update_itx, msg))
    );

    Interface::new(vec![initiate_download, abort_download, update_report], vec![], vec![])
}

fn send(itx: &Sender<Interpret>, cmd: Command) {
    itx.send(Interpret { command: cmd, response_tx: None });
}

fn handle_initiate_download(itx: &Sender<Interpret>, msg: &mut Message) -> MethodResult {
    let sender = try!(msg.sender().map(|s| s.to_string()).ok_or(dbus::missing_arg()));
    debug!("handle_initiate_download: sender={:?}, msg={:?}", sender, msg);

    let mut args = msg.get_items().into_iter();
    let arg_id   = try!(args.next().ok_or(dbus::missing_arg()));
    let update_id: &String = try!(FromMessageItem::from(&arg_id).or(Err(dbus::malformed_arg())));
    send(itx, Command::AcceptUpdates(vec![update_id.clone()]));

    Ok(vec![])
}

fn handle_abort_download(itx: &Sender<Interpret>, msg: &mut Message) -> MethodResult {
    let sender = try!(msg.sender().map(|s| s.to_string()).ok_or(dbus::missing_arg()));
    debug!("handle_abort_download: sender={:?}, msg={:?}", sender, msg);

    let mut args = msg.get_items().into_iter();
    let arg = try!(args.next().ok_or(dbus::missing_arg()));
    let update_id: &String = try!(FromMessageItem::from(&arg).or(Err(dbus::malformed_arg())));
    send(itx, Command::AbortUpdates(vec![update_id.clone()]));

    Ok(vec![])
}

fn handle_update_report(itx: &Sender<Interpret>, msg: &mut Message) -> MethodResult {
    let sender   = try!(msg.sender().map(|s| s.to_string()).ok_or(dbus::missing_arg()));
    debug!("handle_update_report: sender ={:?}, msg ={:?}", sender, msg);
    let mut args = msg.get_items().into_iter();

    let id_arg = try!(args.next().ok_or(dbus::missing_arg()));
    let update_id: &String = try!(FromMessageItem::from(&id_arg).or(Err(dbus::malformed_arg())));

    let results_arg = try!(args.next().ok_or(dbus::missing_arg()));
    let msgs: &Vec<MessageItem> = try!(FromMessageItem::from(&results_arg).or(Err(dbus::malformed_arg())));
    let results = try!(msgs.into_iter()
                       .map(|item| -> Result<OperationResult, ()> { FromMessageItem::from(item) })
                       .collect::<Result<Vec<OperationResult>, ()>>()
                       .or(Err(dbus::malformed_arg()))
    );
    send(itx, Command::SendUpdateReport(Some(UpdateReport::new(update_id.clone(), results))));

    Ok(vec![])
}
