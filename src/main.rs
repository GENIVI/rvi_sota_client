extern crate env_logger;
extern crate getopts;
extern crate ota_plus_client;

use getopts::Options;
use std::env;

use ota_plus_client::{config, connect, read_interpret};


fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("l", "loop", "enter testing loop");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m)  => m,
        Err(e) => panic!(e.to_string())
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    env_logger::init().unwrap();

    let cfg_file = env::var("OTA_PLUS_CLIENT_CFG")
        .unwrap_or("/opt/ats/ota/etc/ota.toml".to_string());
    let client = connect::OtaClient::new(config::parse_config(&cfg_file));
    client.check_for_update();

    if matches.opt_present("l") {
        read_interpret::read_interpret_loop();
    }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}
