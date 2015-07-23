extern crate sota_client;
extern crate url;

use sota_client::rvi;

use std::sync::mpsc::channel;
use std::thread;
use url::Url;

fn main() {
    let rvi_edge = rvi::RviServiceEdge::new(
        Url::parse("http://localhost:8901").unwrap(),
        Url::parse("http://localhost:18901").unwrap());
    rvi_edge.register_service("/sota/notify");
    rvi_edge.register_service("/sota/start");
    rvi_edge.register_service("/sota/chunk");
    rvi_edge.register_service("/sota/finish");

    let (tx, rx) = channel();
    let txc = tx.clone();
    thread::spawn(move || {
        rvi_edge.start(rvi::RviServiceHandler::new(txc));
    });

    loop {
        let e = rx.recv().unwrap();
        match (e.service_name.as_ref(), e.params) {
            ("/sota/notify", rvi::MessageEventParams::Notify(p)) => {
                println!("%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%");
                println!("%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%");
                println!("New package available: {}", p.package);
                println!("%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%");
                println!("%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%");
            },
            _ => {}
        }
    }
}
