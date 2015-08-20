extern crate rustc_serialize;

use std::fs::{OpenOptions, File};
use std::io::{SeekFrom, Seek, Write};
use std::path::Path;

use rustc_serialize::base64::FromBase64;

/// Create a writable file descriptor in /tmp/, with name `package_name`
/// Deletes any file with the same name first.
fn create_package_fd(package_name: &str) -> File {
    let prefix: String = "/tmp/".to_string();
    let path_string = prefix + package_name;
    let path = Path::new(&path_string);

    return OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .unwrap(); // TODO: error handling
}


pub struct PackageFile {
    fd: File,
    chunk_size: i32,
    total_size: i32,
    retry_count: i32,
    finished: bool,
    chunk_count: i32
}


impl PackageFile {
    pub fn new(package_name: &str,
               retry_count: i32) -> PackageFile {

        return PackageFile {
            fd: create_package_fd(package_name),
            chunk_size: 0,
            total_size: 0,
            retry_count: retry_count,
            finished: false,
            chunk_count: 0
        }

    }

    /// (Re)Start this package transfer with a new chunk_size and total_size.
    pub fn start(&mut self, chunk_size: i32, total_size: i32) {
        self.chunk_size = chunk_size;
        self.total_size = total_size;
        self.finished = false;
        self.retry_count = self.retry_count - 1;
        assert!(self.retry_count >= 0); // TODO: ping back to server instead of causing panic
    }
    
    /// Check if this package is marked as finished
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Mark this package as finished, return whether all chunks are processed
    pub fn finish(&mut self) -> bool {
        self.finished = true;
        return (self.chunk_size * self.chunk_count) >= self.total_size
    }

    /// Write a chunk with index `index` to the package file
    /// Calculates to position to save to from the index
    pub fn write_chunk(&mut self, encoded_msg: &str, index: i32) {
        let offset: u64 = (self.chunk_size * index) as u64;
        let decoded_msg = encoded_msg.from_base64();

        match decoded_msg {
            Result::Ok(decoded_msg) => {
                // TODO: this is slow, rather use a buffered writer and flush on finish?
                // TODO: error handling (ping back server) on out of space and simliar
                let _ = self.fd.seek(SeekFrom::Start(offset));
                let _ = self.fd.write_all(&decoded_msg);
                let _ = self.fd.flush();

                self.chunk_count = self.chunk_count + 1;
            },
            Result::Err(error) => {
                println!("Could not decode message. Dropping.");
                println!("{}", error);
                panic!()
                // TODO: ping back to server to restart/resend the transmission instead of causing panic
            }
        }
    }
}
