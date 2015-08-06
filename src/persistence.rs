extern crate rustc_serialize;

use std::fs::{OpenOptions, File};
use std::io::{SeekFrom, Seek, Write};
use std::path::Path;

use rustc_serialize::base64::FromBase64;

pub fn create_package_fd(package_name: &str) -> File {
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

/// Write a base64 encoded chunk with offset `offset` to the provided File
// TODO: this is slow, rather use a buffered writer and flush on finish?
pub fn write_chunk(encoded_msg: &str, offset: u64, mut fd: &File) {
    let decoded_msg = encoded_msg.from_base64().unwrap(); // TODO: error handling

    // TODO: error checking
    fd.seek(SeekFrom::Start(offset));
    fd.write_all(&decoded_msg);
    fd.flush();
}
