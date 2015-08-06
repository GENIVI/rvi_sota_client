extern crate sota_client;
extern crate url;

use sota_client::rvi;
use sota_client::persistence::PackageFile;

use std::env;
use std::sync::mpsc::channel;
use std::thread;
use url::Url;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::Mutex;

// TODO: Add error handling, remove `unwrap()`
/// Start a SOTA client service listenenig on the provided address/port combinations
fn main() {
    let mut args = env::args();
    args.next();
    let rvi_url = args.next().unwrap_or(
        "http://localhost:8901".to_string());
    let edge_url = args.next().unwrap_or(
        "http://localhost:18901".to_string());

    let rvi_edge = rvi::RviServiceEdge::new(
        Url::parse(rvi_url.as_ref()).unwrap(),
        Url::parse(edge_url.as_ref()).unwrap());
    rvi_edge.register_service("/sota/notify");
    rvi_edge.register_service("/sota/start");
    rvi_edge.register_service("/sota/chunk");
    rvi_edge.register_service("/sota/finish");

    let (tx, rx) = channel();
    let txc = tx.clone();
    thread::spawn(move || {
        rvi_edge.start(rvi::RviServiceHandler::new(txc));
    });

    // TODO: concurrency?
    // TODO: error handling
    // TODO: tests
    let mut packages = HashMap::new();

    loop {
        // TODO: abstract away the HashMap unwrapping
        let e = rx.recv().unwrap();
        match (e.service_name.as_ref(), e.params) {
            ("/sota/notify", rvi::MessageEventParams::Notify(p)) => {
                println!("New package available: {} with id {}", p.package, e.message_id);
                let pfile = PackageFile::new(&(p.package), 0, p.retry);
                packages.insert(e.message_id, Mutex::new(pfile));
            },
            ("/sota/start", rvi::MessageEventParams::Start(p)) => {
                match packages.entry(e.message_id) {
                    Entry::Occupied(mut entry) => {
                        entry.get_mut().lock().unwrap()
                            .update_chunk_size(p.chunk_size);
                    }
                    Entry::Vacant(_) => {
                        println!("Dropping unnotified start message with id: {}", e.message_id);
                    }
                }
            }
            ("/sota/chunk", rvi::MessageEventParams::Chunk(p)) => {
                match packages.entry(e.message_id) {
                    Entry::Occupied(mut entry) => {
                        entry.get_mut().lock().unwrap()
                            .write_chunk(&(p.msg), p.index);
                    }
                    Entry::Vacant(_) => {
                        println!("Dropping unnotified chunk message with id: {}", e.message_id);
                    }
                }
            },
            ("/sota/finish", rvi::MessageEventParams::Finish(_)) => {
                match packages.entry(e.message_id) {
                    Entry::Occupied(mut entry) => {
                        let _ = entry.get_mut().lock().unwrap();
                    }
                    Entry::Vacant(_) => {
                        println!("Dropping unnotified finish message with id: {}", e.message_id);
                    }
                }
                println!("Removing pkg id {}", e.message_id);
                packages.remove(&e.message_id);
            },
            _ => {}
        }
    }
}
