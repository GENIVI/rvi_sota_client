// TODO: proper argument parsing
//       See crates docopt or getopts
extern crate sota_client;
#[macro_use] extern crate log;
extern crate env_logger;

use std::env;
use sota_client::configuration::Configuration;
use sota_client::main_loop;

#[cfg_attr(test, allow(dead_code))]
fn main() {
    env_logger::init().unwrap();

    let conf_file = Configuration::default_path();
    let configuration = match Configuration::read(&conf_file) {
        Ok(value) => value,
        Err(e) => {
            error!("Couldn't parse configuration file at {}: {}", conf_file, e);
            std::process::exit(126);
        }
    };

    let mut args = env::args();
    args.next();

    let rvi_url: String = args.next().unwrap_or(
        configuration.client.rvi_url.clone().unwrap_or(
        "http://localhost:8901".to_string()));
    let edge_url: String = args.next().unwrap_or(
        configuration.client.edge_url.clone().unwrap_or(
        "localhost:18901".to_string()));

    main_loop::start(&configuration, rvi_url, edge_url);
}
