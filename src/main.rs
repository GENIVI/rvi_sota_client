extern crate sota_client;
extern crate url;

use sota_client::rvi;
use sota_client::persistence;

use std::env;
use std::sync::mpsc::channel;
use std::thread;
use url::Url;
use std::fs::File;

/// TODO: Add error handling, remove `unwrap()`

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
    let mut chunk_size: i32 = 0;
    let mut package_fd: File = File::create("dummy").unwrap();

    loop {
        let e = rx.recv().unwrap();
        match (e.service_name.as_ref(), e.params) {
            ("/sota/notify", rvi::MessageEventParams::Notify(p)) => {
                println!("New package available: {}", p.package);
            },
            ("/sota/start", rvi::MessageEventParams::Start(p)) => {
                package_fd = persistence::create_package_fd(&(p.package));
                chunk_size = p.chunk_size.clone();
            }
            ("/sota/chunk", rvi::MessageEventParams::Chunk(p)) => {
                let offset = chunk_size * p.index;
                persistence::write_chunk(&(p.msg), offset as u64, &package_fd);
            },
            _ => {}
        }
    }
}
