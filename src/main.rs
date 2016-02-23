extern crate ota_plus_client;
extern crate env_logger;

use std::env;

use ota_plus_client::{config, connect};

fn main() {
    env_logger::init().unwrap();

    let cfg_file = env::var("OTA_PLUS_CLIENT_CFG").unwrap_or("/opt/ats/ota/etc/ota.toml".to_string());
    let client = connect::OtaClient::new(config::parse_config(&cfg_file));
    client.check_for_update();
}
