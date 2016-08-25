use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;


fn run_client(args: &[&str]) -> String {
    let output = Command::new(format!("{}/sota_client", bin_dir()))
        .args(args)
        .output()
        .unwrap_or_else(|err| panic!("failed to execute child: {}", err));
    String::from_utf8(output.stdout).unwrap()
}

fn run_client_with_config(filename: &str, args: &[&str], config: &str) -> String {
    let mut file = File::create(filename.to_string()).expect("couldn't create test config file");
    let _        = file.write_all(config.as_bytes()).unwrap();
    let _        = file.flush().unwrap();
    let arg      = "--config=".to_string() + filename;
    let mut args = args.to_vec();
    args.push(&arg);
    run_client(&args)
}

fn bin_dir() -> String {
    let out_dir = env::var("OUT_DIR").unwrap();
    let bin_dir = Path::new(&out_dir)
        .parent().unwrap()
        .parent().unwrap()
        .parent().unwrap();
    String::from(bin_dir.to_str().unwrap())
}


#[test]
fn help() {
    assert_eq!(run_client(&["-h"]),
               format!(r#"Usage: {}/sota_client [options]

Options:
    -h, --help          print this help menu
        --config PATH   change config path
        --auth-server URL
                        change the auth server
        --auth-client-id ID
                        change the auth client id
        --auth-client-secret SECRET
                        change the auth client secret
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
        --device-system-info PATH
                        change the system information command
        --gateway-console BOOL
                        toggle the console gateway
        --gateway-dbus BOOL
                        toggle the dbus gateway
        --gateway-http BOOL
                        toggle the http gateway
        --gateway-rvi BOOL
                        toggle the rvi gateway
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
    assert_eq!(run_client(&["--config", "tests/sota.toml", "--core-server", "bad-url"]),
               "Invalid core-server URL: Url parse error: relative URL without a base\n")
}

#[test]
fn bad_section() {
    assert_eq!(run_client_with_config("/tmp/sota-test-config-1", &[""], "[foo]\n"),
               "Parse error: invalid section: core\n")
}

#[test]
fn bad_toml() {
    assert_eq!(run_client_with_config("/tmp/sota-test-config-2", &[""], "auth]"),
               "Toml parser errors: [ParserError { lo: 4, hi: 5, desc: \"expected `=`, but found `]`\" }]\n")
}

#[test]
fn bad_path_dir() {
    assert_eq!(run_client(&["--config=/"]),
               "IO error: Is a directory (os error 21)\n")
}
