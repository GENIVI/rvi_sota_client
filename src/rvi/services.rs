use chan;
use chan::Sender;
use rustc_serialize::{json, Decodable, Encodable};
use std::thread;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use time;

use datatype::{ChunkReceived, Event, InstalledSoftware, RpcRequest, RpcOk, RpcErr,
               RviConfig, StartDownload, UpdateReport, UpdateRequestId, Url};
use super::parameters::{Abort, Chunk, Finish, Notify, Parameter, Report, Start};
use super::transfers::Transfers;


#[derive(Clone)]
pub struct Services {
    pub remote:    Arc<Mutex<RemoteServices>>,
    pub sender:    Arc<Mutex<Sender<Event>>>,
    pub transfers: Arc<Mutex<Transfers>>,
}

impl Services {
    pub fn new(rvi_cfg: RviConfig, device_id: String, sender: Sender<Event>) -> Self {
        let transfers = Arc::new(Mutex::new(Transfers::new(rvi_cfg.storage_dir)));
        rvi_cfg.timeout.map_or_else(|| info!("Transfers will never time out."), |timeout| {
            info!("Transfers timeout after {} seconds.", timeout);
            let transfers = transfers.clone();
            thread::spawn(move || {
                let tick = chan::tick(Duration::from_secs(1));
                loop {
                    let _ = tick.recv();
                    let mut transfers = transfers.lock().unwrap();
                    transfers.prune(time::get_time().sec, timeout);
                }
            });
        });

        Services {
            remote:    Arc::new(Mutex::new(RemoteServices::new(device_id, rvi_cfg.client))),
            sender:    Arc::new(Mutex::new(sender)),
            transfers: transfers,
        }
    }

    pub fn register_services<F: Fn(&str) -> String>(&mut self, register: F) {
        let _ = register("/sota/notify");
        let mut remote = self.remote.lock().unwrap();
        remote.local   = Some(LocalServices {
            start_url:    register("/sota/start"),
            chunk_url:    register("/sota/chunk"),
            abort_url:    register("/sota/abort"),
            finish_url:   register("/sota/finish"),
            packages_url: register("/sota/getpackages")
        });
    }

    pub fn handle_service(&self, service: &str, id: u64, msg: &str) -> Result<RpcOk<i32>, RpcErr> {
        match service {
            "/sota/notify"      => self.handle_message::<Notify>(id, msg),
            "/sota/start"       => self.handle_message::<Start>(id, msg),
            "/sota/chunk"       => self.handle_message::<Chunk>(id, msg),
            "/sota/finish"      => self.handle_message::<Finish>(id, msg),
            "/sota/getpackages" => self.handle_message::<Report>(id, msg),
            "/sota/abort"       => self.handle_message::<Abort>(id, msg),
            _                   => Err(RpcErr::invalid_request(id, format!("unknown service: {}", service)))
        }
    }

    fn handle_message<P>(&self, id: u64, msg: &str) -> Result<RpcOk<i32>, RpcErr>
        where P: Parameter + Encodable + Decodable
    {
        let request = try!(json::decode::<RpcRequest<RviMessage<P>>>(&msg).map_err(|err| {
            error!("couldn't decode message: {}", err);
            RpcErr::invalid_params(id, format!("couldn't decode message: {}", err))
        }));
        let event = try!(request.params.parameters[0].handle(&self.remote, &self.transfers).map_err(|err| {
            error!("couldn't handle parameters: {}", err);
            RpcErr::unspecified(request.id, format!("couldn't handle parameters: {}", err))
        }));
        event.map(|ev| self.sender.lock().unwrap().send(ev));
        Ok(RpcOk::new(request.id, None))
    }
}


pub struct RemoteServices {
    pub device_id:  String,
    pub rvi_client: Url,
    pub local:      Option<LocalServices>,
    pub backend:    Option<BackendServices>
}

impl RemoteServices {
    pub fn new(device_id: String, rvi_client: Url) -> RemoteServices {
        RemoteServices { device_id: device_id, rvi_client: rvi_client, local: None, backend: None }
    }

    fn send_message<E: Encodable>(&self, body: E, addr: &str) -> Result<String, String> {
        RpcRequest::new("message", RviMessage::new(addr, vec![body], 60)).send(self.rvi_client.clone())
    }

    pub fn send_start_download(&self, update_id: UpdateRequestId) -> Result<String, String> {
        let backend = try!(self.backend.as_ref().ok_or("BackendServices not set"));
        let local   = try!(self.local.as_ref().ok_or("LocalServices not set"));
        let start   = StartDownload { device_id: self.device_id.clone(), update_id: update_id, local: local.clone() };
        self.send_message(start, &backend.start_url)
    }

    pub fn send_chunk_received(&self, chunk: ChunkReceived) -> Result<String, String> {
        let backend = try!(self.backend.as_ref().ok_or("BackendServices not set"));
        self.send_message(chunk, &backend.ack_url)
    }

    pub fn send_update_report(&self, report: UpdateReport) -> Result<String, String> {
        let backend = try!(self.backend.as_ref().ok_or("BackendServices not set"));
        let result  = UpdateReportResult { device_id: self.device_id.clone(), report: report };
        self.send_message(result, &backend.report_url)
    }

    pub fn send_installed_software(&self, installed: InstalledSoftware) -> Result<String, String> {
        let backend = try!(self.backend.as_ref().ok_or("BackendServices not set"));
        let result  = InstalledSoftwareResult { device_id: self.device_id.clone(), installed: installed };
        self.send_message(result, &backend.packages_url)
    }
}


#[derive(Clone, RustcDecodable, RustcEncodable)]
pub struct LocalServices {
    pub start_url:    String,
    pub chunk_url:    String,
    pub abort_url:    String,
    pub finish_url:   String,
    pub packages_url: String,
}

#[derive(Clone, RustcDecodable, RustcEncodable)]
pub struct BackendServices {
    pub start_url:    String,
    pub ack_url:      String,
    pub report_url:   String,
    pub packages_url: String
}


#[derive(RustcDecodable, RustcEncodable)]
struct UpdateReportResult {
    pub device_id: String,
    pub report:    UpdateReport
}

#[derive(RustcDecodable, RustcEncodable)]
struct InstalledSoftwareResult {
    device_id: String,
    installed: InstalledSoftware
}


#[derive(RustcDecodable, RustcEncodable)]
pub struct RviMessage<E: Encodable> {
    pub service:    String,
    pub parameters: Vec<E>,
    pub timeout:    i64,
}

impl<E: Encodable> RviMessage<E> {
    pub fn new(service: &str, parameters: Vec<E>, expire_in: i64) -> RviMessage<E> {
        RviMessage {
            service:    service.to_string(),
            parameters: parameters,
            timeout:    (time::get_time() + time::Duration::seconds(expire_in)).sec,
        }
    }
}
