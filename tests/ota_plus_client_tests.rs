extern crate tempfile;

use std::env;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use tempfile::NamedTempFile;


fn bin_dir() -> String {
    let out_dir = env::var("OUT_DIR").unwrap();
    let bin_dir = Path::new(&out_dir)
        .parent().unwrap()
        .parent().unwrap()
        .parent().unwrap();
    String::from(bin_dir.to_str().unwrap())
}

fn client(args: &[&str]) -> String {
    let output = Command::new(format!("{}/sota_client", bin_dir()))
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("failed to execute child: {}", e));
    String::from_utf8(output.stdout).unwrap()
}

fn client_with_config(args: &[&str], cfg: &str) -> String {
    let mut file = NamedTempFile::new().unwrap();
    let _        = file.write_all(cfg.as_bytes()).unwrap();
    let arg      = "--config=".to_string() + file.path().to_str().unwrap();
    let mut args = args.to_vec();
    args.push(&arg);
    client(&args)
}


#[test]
fn help() {
    assert_eq!(client(&["-h"]),
               format!(r#"Usage: {}/sota_client [options]

Options:
    -h, --help          print this help menu
        --config PATH   change config path
        --auth-server URL
                        change the auth server
        --auth-client-id ID
                        change the auth client id
        --auth-secret SECRET
                        change the auth secret
        --auth-credentials-file PATH
                        change the auth credentials file
        --core-server URL
                        change the core server
        --dbus-name NAME
                        change the dbus registration name
        --dbus-path PATH
                        change the dbus path
        --dbus-interface INTERFACE
                        change the dbus interface name
        --dbus-software-manager NAME
                        change the dbus software manager name
        --dbus-software-manager-path PATH
                        change the dbus software manager path
        --dbus-timeout TIMEOUT
                        change the dbus installation timeout
        --device-uuid UUID
                        change the device uuid
        --device-vin VIN
                        change the device vin
        --device-packages-dir PATH
                        change downloaded directory for packages
        --device-package-manager MANAGER
                        change the package manager
        --device-polling-interval INTERVAL
                        change the package polling interval
        --device-certificates-path PATH
                        change the OpenSSL CA certificates file
        --gateway-console BOOL
                        toggle the console gateway
        --gateway-dbus BOOL
                        toggle the dbus gateway
        --gateway-http BOOL
                        toggle the http gateway
        --gateway-websocket BOOL
                        toggle the websocket gateway
        --rvi-client URL
                        change the rvi client URL
        --rvi-edge URL  change the exposed rvi edge URL
        --rvi-storage-dir PATH
                        change the rvi storage directory
        --rvi-timeout TIMEOUT
                        change the rvi timeout

"#, bin_dir()));
}

#[test]
fn bad_ota_server_url() {
    assert_eq!(client(&["--core-server", "apa"]),
               "Invalid core-server URL: Url parse error: relative URL without a base\n")
}

#[test]
fn bad_section() {
    assert_eq!(client_with_config(&[""], "[foo]\n"),
               "parse_section, invalid section: core\n")
}

#[test]
fn bad_toml() {
    assert_eq!(client_with_config(&[""], "auth]"),
               "Toml parser errors: [ParserError { lo: 4, hi: 5, desc: \"expected `=`, but found `]`\" }]\n")
}

#[test]
fn bad_path_dir() {
    assert_eq!(client(&["--config=/"]),
               "IO error: Is a directory (os error 21)\n")
}
