extern crate libotaplus;
extern crate env_logger;
extern crate getopts;

use getopts::Options;
use std::env;
use std::process::exit;

use libotaplus::{config, read_interpret};
use libotaplus::config::Config;
use libotaplus::read_interpret::ReplEnv;
use libotaplus::ota_plus::{Client as OtaClient};
use libotaplus::auth_plus::{Client as AuthClient};
use libotaplus::package_manager::{PackageManager, Dpkg};
use libotaplus::error::Error;

fn main() {

    env_logger::init().unwrap();

    let config_file = env::var("OTA_PLUS_CLIENT_CFG")
        .unwrap_or("/opt/ats/ota/etc/ota.toml".to_string());

    let config = config::load_config(&config_file)
        .unwrap_or_else(|err| {
            println!("{} (continuing with the default config)", err);
            return Config::default();
        });

    do_stuff(handle_flags(config));

}

fn do_stuff(config: Config) {

    fn post_installed_packages<M>(client: OtaClient, manager: M) -> Result<(), Error>
        where M: PackageManager {
            manager.installed_packages().and_then(|pkgs| client.post_packages(pkgs))
        }

    fn build_ota_client(config: Config) -> Result<OtaClient, Error> {
        AuthClient::new(config.auth.clone()).authenticate().map(|token| {
            OtaClient::new(token, config.ota.clone())
        })
    }

    let pkg_manager = Dpkg::new();
    let pkg_manager_clone = pkg_manager.clone();

    let _ = build_ota_client(config.clone()).and_then(|client| {
        post_installed_packages(client, pkg_manager)
    }).map(|_| {
        print!("Installed packages were posted successfully.");
    }).map_err(|e| {
        print!("{}", e);
    });

    if config.test.interpret {
        read_interpret::read_interpret_loop(ReplEnv::new(pkg_manager_clone));
    }
}

fn handle_flags(config: Config) -> Config {

    fn print_usage(program: &str, opts: Options) {
        let brief = format!("Usage: {} [options]", program);
        print!("{}", opts.usage(&brief));
    }

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
        exit(1);
    }

    if matches.opt_present("l") {
        let mut config = config;
        config.test.interpret = true;
        return config
    }
    return config
}
