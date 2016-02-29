use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::PathBuf;

use hyper::Url;
use rustc_serialize::Decodable;
use toml;

#[derive(RustcDecodable)]
pub struct AuthConfig {
    pub server: Url,
    pub client_id: String,
    pub secret: String
}

#[derive(RustcDecodable)]
pub struct OtaConfig {
    pub server: Url,
    pub vin: String
}

fn read_config(path: &str) -> toml::Table {
    OpenOptions::new().open(PathBuf::from(path))
        .map_err(|e| error!("Cannot open config file: {}, error: {}", path, e))
        .and_then(|mut f| {
            let mut buf = String::new();
            f.read_to_string(&mut buf)
                .map(|_| buf)
                .map_err(|e| error!("Cannot read config file: {}, error: {}", path, e)) })
        .and_then(|s| {
            toml::Parser::new(&s).parse()
                .ok_or(())
                .map_err(|_| error!("Cannot parse config file: {}", path)) })
        .unwrap()
}

fn parse_sect<T: Decodable>(cfg: &toml::Table, sect: &str) -> T {
    cfg.get(sect)
        .and_then(|c| toml::decode::<T>(c.clone()) )
        .ok_or_else(|| error!("Invalid section in config file: {}", sect))
        .unwrap()
}

pub fn parse_config(path: &str) -> (AuthConfig, OtaConfig) {
    let cfg = read_config(path);
    (parse_sect(&cfg, "auth"), parse_sect(&cfg, "ota"))
}

