extern crate env_logger;
extern crate getopts;
extern crate hyper;
#[macro_use] extern crate libotaplus;

use getopts::Options;
use hyper::Url;
use std::env;

use libotaplus::auth_plus::authenticate;
use libotaplus::datatype::config;
use libotaplus::datatype::Config;
use libotaplus::datatype::Error;
use libotaplus::datatype::PackageManager as PackageManagerType;
use libotaplus::http_client::HttpClient;
use libotaplus::ota_plus::{post_packages, get_package_updates, download_package_update};
use libotaplus::package_manager::{PackageManager, Dpkg};
use libotaplus::read_interpret::ReplEnv;
use libotaplus::read_interpret;


fn main() {

    env_logger::init().unwrap();

    let config = build_config();

    match worker::<hyper::Client>(&config, config.ota.package_manager.build()) {
        Ok(()) => {},
        Err(e) => exit!("{}", e),
    }

    if config.test.looping {
        read_interpret::read_interpret_loop(ReplEnv::new(Dpkg));
    }

}

fn worker<C: HttpClient>(config: &Config, pkg_manager: &PackageManager) -> Result<(), Error> {

    println!("Trying to acquire access token.");
    let token = try!(authenticate::<C>(&config.auth));

    println!("Asking package manager what packages are installed on the system.");
    let pkgs = try!(pkg_manager.installed_packages());

    println!("Letting the OTA server know what packages are installed.");
    try!(post_packages::<C>(&token, &config.ota, &pkgs));

    println!("Fetching possible new package updates.");
    let updates = try!(get_package_updates::<C>(&token, &config.ota));

    let updates_len = updates.iter().len();
    println!("Got {} new updates. Downloading...", updates_len);

    let mut paths = Vec::with_capacity(updates_len);

    for update in &updates {
        let path = try!(download_package_update::<C>(&token, &config.ota, update)
                        .map_err(|e| Error::ClientError(
                            format!("Couldn't download update {:?}: {}", update, e))));
        paths.push(path);
    }

    for path in &paths {
        println!("Installing package at {:?}...", path);
        try!(pkg_manager.install_package(path.as_path()));
        println!("Installed.");
    }

    println!("Installed packages were posted successfully.");

    return Ok(())

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
    opts.optopt("", "ota-packages-dir",
                "change downloaded directory for packages", "PATH");
    opts.optopt("", "ota-package-manager",
                "change package manager", "MANAGER");
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

    if let Some(path) = matches.opt_str("ota-packages-dir") {
        config.ota.packages_dir = path;
    }

    if let Some(s) = matches.opt_str("ota-package-manager") {
        config.ota.package_manager = match s.to_lowercase().as_str() {
            "dpkg" => PackageManagerType::Dpkg,
            "rpm"  => PackageManagerType::Rpm,
            "test" => PackageManagerType::Test,
            s      => exit!("Invalid package manager: {}", s)
        }
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
