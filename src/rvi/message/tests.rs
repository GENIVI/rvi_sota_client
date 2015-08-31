use super::*;
use std::sync::Mutex;
use std::collections::HashMap;
use persistence::PackageFile;
use std::env::temp_dir;
use std::fs::OpenOptions;
use std::io::prelude::*;
use time;

const RETRY_COUNT: i32 = 10;
const TOTAL_SIZE: i32 = 5;
const CHUNK_SIZE: i32 = 131072;
const ID: u32 = 456;

static MSG: &'static str = "dGVzdAo="; // 'test' in base 64

macro_rules! use_pending {
    () => { Mutex::new(HashMap::<String, i32>::new()) }
}
macro_rules! use_transfers {
    () => { Mutex::new(HashMap::<u32, PackageFile>::new()) }
}
macro_rules! use_package {
    () => { time::precise_time_ns().to_string() + "-rust-test.rpm" }
}
macro_rules! use_notify {
    ($package:ident) => {
        NotifyParams {
            retry: RETRY_COUNT,
            package: $package.clone()
        }
    }
}
macro_rules! use_start {
    ($package:ident) => {
        StartParams {
                id: ID,
                total_size: TOTAL_SIZE,
                chunk_size: CHUNK_SIZE,
                package: $package.clone()
        }
    }
}
macro_rules! use_chunk {
    () => {
        ChunkParams {
            id: ID,
            index: 0,
            msg: MSG.to_string()
        }
    }
}
macro_rules! use_finish {
    () => { FinishParams { id: ID } }
}
macro_rules! assert_pending {
    ($pending:ident, $package:ident) => {{
        assert_eq!($pending.lock().unwrap().remove(&$package).unwrap(), RETRY_COUNT);
        assert!($pending.lock().unwrap().is_empty());
    }}
}
macro_rules! assert_transfers {
    ($transfers:ident, $package:ident) => {{
        let pfile = $transfers.lock().unwrap().remove(&ID).unwrap();
        assert_eq!(pfile.package_name(), $package);
        assert_eq!(pfile.total_size(), TOTAL_SIZE);
        assert_eq!(pfile.chunk_size(), CHUNK_SIZE);
        assert!($transfers.lock().unwrap().is_empty());
    }}
}
macro_rules! open_file {
    ($package:ident) => {{
        let mut path = temp_dir();
        path.push(&$package);

        OpenOptions::new()
            .write(false)
            .create(false)
            .truncate(false)
            .open(path)
    }}
}
macro_rules! assert_file_written {
    ($package:ident) => {{
        let mut file = open_file!($package).unwrap();

        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();

        assert_eq!(content, "test\n".to_string());
    }}
}
macro_rules! assert_file_not_written {
    ($package:ident) => {{
        match open_file!($package) {
            Ok(_) => {
                panic!("File {} exists!", $package);
            },
            Err(_) => {}
        }
    }}
}

#[test]
fn notify_handler() {
    let pending = use_pending!();
    let transfers = use_transfers!();
    let package = use_package!();
    let notify = use_notify!(package);

    println!("notify returns true");
    assert!(notify.handle(&pending, &transfers));

    println!("notify sets the correct retry count");
    assert_pending!(pending, package);
}

#[test]
fn start_handler() {
    let pending = use_pending!();
    let transfers = use_transfers!();
    let package = use_package!();
    let start = use_start!(package);
    let notify = use_notify!(package);

    // add a valid package to pending
    assert!(notify.handle(&pending, &transfers));

    println!("start returns true on valid transfer id");
    assert!(start.handle(&pending, &transfers));

    println!("start adds exactly one transfer to the transfers map");
    assert_transfers!(transfers, package);

    println!("start does not tamper with the pending map");
    assert_pending!(pending, package);
}

#[test]
fn start_handler_unnotified() {
    let pending = use_pending!();
    let transfers = use_transfers!();
    let package = use_package!();
    let start = use_start!(package);

    println!("start returns false on invalid package");
    assert!(! start.handle(&pending, &transfers));

    println!("an invalid package is not added to the transfers map");
    assert!(transfers.lock().unwrap().is_empty());

    println!("an invalid package is not added to the pending map");
    assert!(pending.lock().unwrap().is_empty());
}

#[test]
fn chunk_handler() {
    let pending = use_pending!();
    let transfers = use_transfers!();
    let package = use_package!();
    let start = use_start!(package);
    let notify = use_notify!(package);
    let chunk = use_chunk!();

    // add a valid package to pending and transfers
    assert!(notify.handle(&pending, &transfers));
    assert!(start.handle(&pending, &transfers));

    println!("chunk returns true on valid transfers");
    assert!(chunk.handle(&pending, &transfers));

    println!("chunk writes a correct chunk to disk");
    assert_file_written!(package);

    println!("chunk does not remove a non-finished file from transfers");
    assert_transfers!(transfers, package);

    println!("chunk does not remove a non-finished file from pending");
    assert_pending!(pending, package)
}

#[test]
fn chunk_handler_unstarted() {
    let pending = use_pending!();
    let transfers = use_transfers!();
    let package = use_package!();
    let notify = use_notify!(package);
    let chunk = use_chunk!();

    // add a valid package to pending but not to transfers
    assert!(notify.handle(&pending, &transfers));

    println!("chunk returns false on unstarted transfers");
    assert!(! chunk.handle(&pending, &transfers));

    println!("chunk doesn't write an incorrect chunk to disk");
    assert_file_not_written!(package);

    println!("chunk does not add a package to transfers");
    assert!(transfers.lock().unwrap().is_empty());

    println!("chunk does not remove a unstarted file from pending");
    assert_pending!(pending, package)
}

#[test]
fn chunk_handler_unnotified() {
    let pending = use_pending!();
    let transfers = use_transfers!();
    let package = use_package!();
    let chunk = use_chunk!();

    println!("chunk returns false on unknown packages");
    assert!(! chunk.handle(&pending, &transfers));

    println!("chunk doesn't write an incorrect chunk to disk");
    assert_file_not_written!(package);

    println!("chunk does not add a package to transfers");
    assert!(transfers.lock().unwrap().is_empty());

    println!("chunk does not add a package to pending");
    assert!(pending.lock().unwrap().is_empty());
}

#[test]
fn chunk_handler_after_finish() {
    let pending = use_pending!();
    let transfers = use_transfers!();
    let package = use_package!();
    let start = use_start!(package);
    let notify = use_notify!(package);
    let chunk = use_chunk!();
    let finish = use_finish!();

    // add a valid package to pending and transfers
    assert!(notify.handle(&pending, &transfers));
    assert!(start.handle(&pending, &transfers));
    // mark the transfer as finished
    assert!(finish.handle(&pending, &transfers));

    println!("chunk returns true on valid, finished transfers");
    assert!(chunk.handle(&pending, &transfers));

    // TODO: figure out how to check for the disk changes, before PackageFile goes out of scope and
    //       cleans the files.
    // println!("chunk writes the correct chunk to disk");
    // assert_file_written!(package);

    println!("chunk removes a finished file from transfers");
    assert!(transfers.lock().unwrap().is_empty());

    println!("chunk removes a finished file from pending");
    assert!(pending.lock().unwrap().is_empty());
}

#[test]
fn finish_handler() {
    let pending = use_pending!();
    let transfers = use_transfers!();
    let package = use_package!();
    let start = use_start!(package);
    let notify = use_notify!(package);
    let chunk = use_chunk!();
    let finish = use_finish!();

    // add a valid package to pending and transfers and finish the download
    assert!(notify.handle(&pending, &transfers));
    assert!(start.handle(&pending, &transfers));
    assert!(chunk.handle(&pending, &transfers));

    println!("finish returns true on valid, finished transfers");
    assert!(finish.handle(&pending, &transfers));

    println!("finish removes the finished file from transfers");
    assert!(transfers.lock().unwrap().is_empty());

    println!("finish removes the finished file from pending");
    assert!(pending.lock().unwrap().is_empty());
}

#[test]
fn finish_handler_not_all_chunks_received() {
    let pending = use_pending!();
    let transfers = use_transfers!();
    let package = use_package!();
    let start = use_start!(package);
    let notify = use_notify!(package);
    let finish = use_finish!();

    // add a valid package to pending and transfers without receiving any chunks
    assert!(notify.handle(&pending, &transfers));
    assert!(start.handle(&pending, &transfers));

    println!("finish returns true on unfinished transfers");
    assert!(finish.handle(&pending, &transfers));
    
    println!("finish marks unfinished transfers as finished instead of removing them");
    let pfile = transfers.lock().unwrap().remove(&ID).unwrap();
    assert_eq!(pfile.package_name(), package);
    assert_eq!(pfile.total_size(), TOTAL_SIZE);
    assert_eq!(pfile.chunk_size(), CHUNK_SIZE);
    assert!(pfile.is_finished());
    assert!(transfers.lock().unwrap().is_empty());

    println!("finish keeps the unfinished file in pending");
    assert_pending!(pending, package);
}

#[test]
fn finish_handler_unstarted() {
    let pending = use_pending!();
    let transfers = use_transfers!();
    let package = use_package!();
    let notify = use_notify!(package);
    let finish = use_finish!();

    // add a valid package to pending only
    assert!(notify.handle(&pending, &transfers));

    println!("finish returns false on unstarted transfers");
    assert!(! finish.handle(&pending, &transfers));

    println!("finish doesn't add anything to transfers");
    assert!(transfers.lock().unwrap().is_empty());

    println!("finish keeps the pending map intact");
    assert_pending!(pending, package);
}

#[test]
fn finish_handler_unnotified() {
    let pending = use_pending!();
    let transfers = use_transfers!();
    let finish = use_finish!();

    println!("finish returns false on unnotified transfers");
    assert!(! finish.handle(&pending, &transfers));

    println!("finish doesn't add anything to transfers");
    assert!(transfers.lock().unwrap().is_empty());

    println!("finish doesn't add anything to pending");
    assert!(pending.lock().unwrap().is_empty());
}
