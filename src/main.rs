extern crate sota_client;
extern crate url;

use sota_client::rvi;
use sota_client::persistence::PackageFile;
use sota_client::unwrap::Unpack;

use std::env;
use std::sync::mpsc::channel;
use std::thread;
use url::Url;
use std::collections::HashMap;
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

    let packages = Mutex::new(HashMap::new());

    loop {
        let e = rx.recv().unwrap();
        let mut packages = packages.lock().unwrap();
        let mut can_remove = false;

        match (e.service_name.as_ref(), e.params) {
            ("/sota/notify", rvi::MessageEventParams::Notify(p)) => {
                println!("New package available: {} with id {}", p.package, e.message_id);

                let pfile = PackageFile::new(&(p.package), p.retry);
                packages.insert(e.message_id, pfile);
            },

            ("/sota/start", rvi::MessageEventParams::Start(p)) => {
                can_remove = packages.entry(e.message_id).unpack_or_println(
                    |package: &mut PackageFile| {
                        package.start(p.chunk_size, p.total_size);
                        false
                    }, e.message_id);
            },

            ("/sota/chunk", rvi::MessageEventParams::Chunk(p)) => {
                can_remove = packages.entry(e.message_id).unpack_or_println(move
                    |package: &mut PackageFile| {
                        package.write_chunk(&(p.msg), p.index);
                        if package.is_finished() {
                            package.finish()
                        } else {
                            false
                        }
                    }, e.message_id);
            },

            ("/sota/finish", rvi::MessageEventParams::Finish(_)) => {
                let id = e.message_id;
                can_remove = packages.entry(id).unpack_or_println(move
                    |package: &mut PackageFile| {
                        println!("Marking package id {} as done", id);
                        package.finish()
                    }, e.message_id);
            },
            _ => {}
        }
        if can_remove {
            packages.remove(&e.message_id);
            println!("Finished package {}", e.message_id)
        }
    }
}
