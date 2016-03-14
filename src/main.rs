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
use libotaplus::ota_plus::{Client as OtaClient};
use libotaplus::auth_plus::{Client as AuthClient};
use libotaplus::package_manager::{PackageManager, Dpkg};


fn main() {

    env_logger::init().unwrap();

    let config = build_config();
    let pkg_manager = Dpkg::new();

    let _ = AuthClient::new(config.auth.clone())
        .authenticate()
        .map(|token| OtaClient::new(token, config.ota.clone()))
        .and_then(|client| pkg_manager.installed_packages()
                  .and_then(|pkgs| client.post_packages(pkgs)))
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


#[cfg(test)]
mod tests {

    use std::ffi::OsStr;
    use std::process::Command;

    fn client<S: AsRef<OsStr>>(args: &[S]) -> String {
        let output = Command::new("target/debug/ota_plus_client")
            .args(args)
            .output()
            .unwrap_or_else(|e| { panic!("failed to execute child: {}", e) });
        return String::from_utf8(output.stdout).unwrap()
    }

    #[test]
    fn help() {

        assert_eq!(client(&["-h"]),
r#"Usage: target/debug/ota_plus_client [options]

Options:
    -h, --help          print this help menu
        --config PATH   change config path
        --auth-server URL
                        change the auth server URL
        --auth-client-id ID
                        change auth client id
        --auth-secret SECRET
                        change auth secret
        --ota-server URL
                        change ota server URL
        --ota-vin VIN   change ota vin
        --test-looping  enable read-interpret test loop

"#);

    }

    #[test]
    fn bad_config_path() {
        assert_eq!(client(&["--config", "apa"]),
                   "Failed to load config: No such file or directory (os error 2)\n");
    }

    #[test]
    fn bad_auth_server_url() {
        assert_eq!(client(&["--auth-server", "apa"]),
                   "Invalid auth-server URL: relative URL without a base\n");
    }

    #[test]
    fn bad_ota_server_url() {
        assert_eq!(client(&["--ota-server", "apa"]),
                   "Invalid ota-server URL: relative URL without a base\n");
    }

    #[test]
    fn no_auth_server_to_connect_to() {
        assert_eq!(client(&[""]),
                   "Authentication error, cannot send token request: connection refused\n");
    }

}
