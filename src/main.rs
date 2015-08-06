extern crate sota_client;
extern crate url;
extern crate rustc_serialize;

use sota_client::rvi;

use std::env;
use std::sync::mpsc::channel;
use std::thread;
use std::fs::{OpenOptions, File};
use std::io::{SeekFrom, Seek, Write};
use std::path::Path;
use url::Url;

use rustc_serialize::base64::FromBase64;

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
                let path = Path::new(&(p.package));
                package_fd = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(path)
                    .unwrap();
                chunk_size = p.chunk_size.clone();
            }
            ("/sota/chunk", rvi::MessageEventParams::Chunk(p)) => {
                let decoded_msg = p.msg.from_base64().unwrap();
                let offset = chunk_size * p.index;

                // TODO: this is slow, rather use a buffered writer and flush on finish?
                package_fd.seek(SeekFrom::Start(offset as u64));
                package_fd.write_all(&decoded_msg);
                package_fd.flush();
            },
            _ => {}
        }
    }
}
