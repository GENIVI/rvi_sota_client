extern crate rustc_serialize;

use std::fs::{OpenOptions, File};
use std::io::{SeekFrom, Seek, Write};
use std::path::Path;

use rustc_serialize::base64::FromBase64;

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

#[allow(dead_code)]
pub struct PackageFile {
    fd: File,
    chunk_size: i32,
    retry_count: i32
}

impl PackageFile {
    pub fn new(package_name: &str,
               chunk_size: i32,
               retry_count: i32) -> PackageFile {

        return PackageFile {
            fd: create_package_fd(package_name),
            chunk_size: chunk_size,
            retry_count: retry_count
        }

    }

    /// Set a new chunk size for this package
    pub fn update_chunk_size(&mut self, chunk_size: i32) {
        self.chunk_size = chunk_size;
    }

    /// Write a base64 encoded chunk with offset `offset` to the provided File
    #[allow(unused_must_use)]
    pub fn write_chunk(&mut self, encoded_msg: &str, index: i32) {
        let offset: u64 = (self.chunk_size * index) as u64;
        let decoded_msg = encoded_msg.from_base64().unwrap(); // TODO: error handling

        // TODO: this is slow, rather use a buffered writer and flush on finish?
        // TODO: error checking
        self.fd.seek(SeekFrom::Start(offset));
        self.fd.write_all(&decoded_msg);
        self.fd.flush();
    }
}
