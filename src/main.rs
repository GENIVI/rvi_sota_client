extern crate env_logger;
extern crate getopts;
extern crate hyper;
#[macro_use] extern crate libotaplus;

use getopts::Options;
use hyper::Url;
use std::env;

use libotaplus::{config, read_interpret};
use libotaplus::config::Config;
use libotaplus::read_interpret::ReplEnv;
use libotaplus::auth_plus::authenticate;
use libotaplus::ota_plus::post_packages;
use libotaplus::package_manager::{PackageManager, Dpkg};

fn main() {

    env_logger::init().unwrap();

    let config = build_config();
    let pkg_manager = Dpkg::new();

    let _ = authenticate::<hyper::Client>(config.auth.clone())
        .and_then(|token| pkg_manager.installed_packages()
                  .and_then(|pkgs| post_packages::<hyper::Client>(token, config.ota.clone(), pkgs)))
        .map(|_| println!("Installed packages were posted successfully."))
        .map_err(|err| println!("{}", err));

    if config.test.looping {
        read_interpret::read_interpret_loop(ReplEnv::new(pkg_manager.clone()));
    }

}

fn build_config() -> Config {

    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help",
                 "print this help menu");
    opts.optopt("", "config",
                "change config path", "PATH");
    opts.optopt("", "auth-server",
                "change the auth server URL", "URL");
    opts.optopt("", "auth-client-id",
                "change auth client id", "ID");
    opts.optopt("", "auth-secret",
                "change auth secret", "SECRET");
    opts.optopt("", "ota-server",
                "change ota server URL", "URL");
    opts.optopt("", "ota-vin",
                "change ota vin", "VIN");
    opts.optflag("", "test-looping",
                 "enable read-interpret test loop");

    let matches = opts.parse(&args[1..])
        .unwrap_or_else(|err| panic!(err.to_string()));

    if matches.opt_present("h") {
        let brief = format!("Usage: {} [options]", program);
        exit!("{}", opts.usage(&brief));
    }

    let mut config_file = env::var("OTA_PLUS_CLIENT_CFG")
        .unwrap_or("/opt/ats/ota/etc/ota.toml".to_string());

    if let Some(path) = matches.opt_str("config") {
        config_file = path;
    }

    let mut config = config::load_config(&config_file)
        .unwrap_or_else(|err| exit!("{}", err));

    if let Some(s) = matches.opt_str("auth-server") {
        match Url::parse(&s) {
            Ok(url)  => config.auth.server = url,
            Err(err) => exit!("Invalid auth-server URL: {}", err)
        }
    }

    if let Some(client_id) = matches.opt_str("auth-client-id") {
        config.auth.client_id = client_id;
    }

    if let Some(secret) = matches.opt_str("auth-secret") {
        config.auth.secret = secret;
    }

    if let Some(s) = matches.opt_str("ota-server") {
        match Url::parse(&s) {
            Ok(url)  => config.ota.server = url,
            Err(err) => exit!("Invalid ota-server URL: {}", err)
        }
    }

    if let Some(vin) = matches.opt_str("ota-vin") {
        config.ota.vin = vin;
    }

    if matches.opt_present("test-looping") {
        config.test.looping = true;
    }

    return config
}

// Hack to build a binary with a predictable path for use in tests/. We
// can remove this when https://github.com/rust-lang/cargo/issues/1924
// is resolved.
#[test]
fn build_binary() {
    let output = std::process::Command::new("cargo")
        .arg("build")
        .output()
        .unwrap_or_else(|e| panic!("failed to execute child: {}", e));

    assert!(output.status.success())
}
