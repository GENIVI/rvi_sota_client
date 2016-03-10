extern crate libotaplus;
extern crate env_logger;
extern crate getopts;

use getopts::Options;
use std::env;

use libotaplus::{config, read_interpret};
use libotaplus::read_interpret::ReplEnv;
use libotaplus::ota_plus::{Client as OtaClient};
use libotaplus::auth_plus::{Client as AuthClient};
use libotaplus::package_manager::{PackageManager, Dpkg};
use libotaplus::error::Error;

fn post_installed_packages<M>(client: OtaClient, manager: M) -> Result<(), Error>
    where M: PackageManager {
    manager.installed_packages().and_then(|pkgs| client.post_packages(pkgs))
}

fn build_ota_client(config: config::Config) -> Result<OtaClient, Error> {
    AuthClient::new(config.auth.clone()).authenticate().map(|token| {
        OtaClient::new(token, config.ota.clone())
    })
}

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
    let config = config::load_config(&cfg_file);

    let pkg_manager = Dpkg::new();
    let pkg_manager_clone = pkg_manager.clone();

    let _ = build_ota_client(config).and_then(|client| {
        post_installed_packages(client, pkg_manager)
    }).map(|_| {
        print!("Installed packages were posted successfully.");
    }).map_err(|e| {
        print!("{}", e);
    });

    if matches.opt_present("l") {
        read_interpret::read_interpret_loop(ReplEnv::new(pkg_manager_clone));
    }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}
