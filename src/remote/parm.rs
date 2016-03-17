use event::UpdateId;
use event::inbound::{InboundEvent, UpdateAvailable, GetInstalledSoftware, DownloadComplete};
use persistence::Transfers;
pub use super::svc::{LocalServices, BackendServices, RemoteServices, ServiceHandler};

use std::result;
use std::sync::Mutex;

#[derive(Debug)]
pub enum Error {
    UnknownPackage,
    IoFailure,
    SendFailure
}
pub type Result =  result::Result<Option<InboundEvent>, Error>;

/// Trait that every message handler needs to implement.
pub trait ParamHandler {
    /// Handle the message.
    /// 
    /// Return a [`Event`](../message/enum.Event.html) to be passed to the
    /// [`main_loop`](../main_loop/index.html) if apropriate.
    fn handle(&self,
              services: &Mutex<RemoteServices>,
              transfers: &Mutex<Transfers>)
        -> Result;
}


/// Type for "Notify" messages.
#[derive(RustcDecodable, Clone)]
pub struct NotifyParams {
    /// A `Vector` of packages, that are available for download.
    pub update_available: UpdateAvailable,
    /// The service URLs, that the SOTA server supports.
    pub services: BackendServices,
}

impl ParamHandler for NotifyParams {
    fn handle(&self,
              services: &Mutex<RemoteServices>,
              _: &Mutex<Transfers>) -> Result {
        let mut services = services.lock().unwrap();
        services.set(self.services.clone());

        Ok(Some(InboundEvent::UpdateAvailable(self.update_available.clone())))
    }
}


/// Type for "Start Transfer" messages.
#[derive(RustcDecodable)]
pub struct StartParams {
    pub update_id: UpdateId,
    /// The amount of chunks this `Transfer` will have.
    pub chunkscount: u64,
    /// The SHA1 checksum of the assembled package.
    pub checksum: String
}

impl ParamHandler for StartParams {
    fn handle(&self,
              services: &Mutex<RemoteServices>,
              transfers: &Mutex<Transfers>) -> Result {
        let services = services.lock().unwrap();
        let mut transfers = transfers.lock().unwrap();

        info!("Starting transfer for update_id {}", self.update_id);

        transfers.push(self.update_id.clone(), self.checksum.clone());
        services.send_chunk_received(
            ChunkReceived {
                update_id: self.update_id.clone(),
                chunks: Vec::new(),
                vin: services.vin.clone() })
            .map_err(|e| {
                error!("Error on sending start ACK: {}", e);
                Error::SendFailure })
            .map(|_| None)
    }
}

/// Encodes the "Chunk Received" message, indicating that a chunk was successfully transferred.
#[derive(RustcEncodable)]
pub struct ChunkReceived {
    /// The transfer to which the transferred chunk belongs.
    pub update_id: UpdateId,
    /// A list of the successfully transferred chunks.
    pub chunks: Vec<u64>,
    /// The VIN of this device.
    pub vin: String
}


/// Type for messages transferring single chunks.
#[derive(RustcDecodable)]
pub struct ChunkParams {
    /// The package transfer this chunk belongs to.
    pub update_id: UpdateId,
    /// The data of the transferred chunk.
    pub bytes: String,
    /// The index of this chunk.
    pub index: u64
}

impl ParamHandler for ChunkParams {
    fn handle(&self,
              services: &Mutex<RemoteServices>,
              transfers: &Mutex<Transfers>) -> Result {
        let services = services.lock().unwrap();
        let mut transfers = transfers.lock().unwrap();
        transfers.get_mut(&self.update_id).map(|t| {
            if t.write_chunk(&self.bytes, self.index) {
                info!("Wrote chunk {} for package {}", self.index, self.update_id);
                services.send_chunk_received(
                    ChunkReceived {
                        update_id: self.update_id.clone(),
                        chunks: t.transferred_chunks.clone(),
                        vin: services.vin.clone() })
                    .map_err(|e| {
                        error!("Error on sending ChunkReceived: {}", e);
                        Error::SendFailure })
                    .map(|_| None)
            } else {
                Err(Error::IoFailure)
            }
        }).unwrap_or_else(|| {
            error!("Couldn't find transfer for update_id {}", self.update_id);
            Err(Error::UnknownPackage)
        })
    }
}


/// Type for "Finish Transfer" messages.
#[derive(RustcDecodable)]
pub struct FinishParams {
    /// The package transfer to finalize.
    pub update_id: UpdateId,
    pub signature: String
}

impl ParamHandler for FinishParams {
    fn handle(&self,
              _: &Mutex<RemoteServices>,
              transfers: &Mutex<Transfers>) -> Result {
        let mut transfers = transfers.lock().unwrap();
        let success = transfers.get(&self.update_id).map(|t| {
            t.assemble_package()
        }).unwrap_or_else(|| {
            error!("Couldn't find transfer for update_id {}", self.update_id);
            false
        });
        if success {
            transfers.remove(&self.update_id);
            info!("Finished transfer of {}", self.update_id);
            Ok(Some(InboundEvent::DownloadComplete(DownloadComplete {
                update_image: String::new(),
                signature: self.signature.clone()
            })))
        } else {
            // TODO: Report transfer error to server
            Err(Error::UnknownPackage)
        }
    }
}


/// Type for "Abort Transfer" messages.
#[derive(RustcDecodable)]
pub struct AbortParams;

impl ParamHandler for AbortParams {
    fn handle(&self,
              _: &Mutex<RemoteServices>,
              transfers: &Mutex<Transfers>) -> Result {
        let mut transfers = transfers.lock().unwrap();
        transfers.clear();
        Ok(None)
    }
}


/// Type for "Get All Packages" messages.
pub type ReportParams = GetInstalledSoftware;

impl ParamHandler for ReportParams {
    fn handle(&self,
              _: &Mutex<RemoteServices>,
              _: &Mutex<Transfers>) -> Result {
        Ok(Some(InboundEvent::GetInstalledSoftware(self.clone())))
    }
}
