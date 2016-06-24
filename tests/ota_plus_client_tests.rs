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
    let output = Command::new(format!("{}/ota_plus_client", bin_dir()))
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
               format!(r#"Usage: {}/ota_plus_client [options]

Options:
    -h, --help          print this help menu
        --repl          enable repl
        --http          enable interaction via http requests
        --no-websocket  disable websocket interaction
        --auth-server URL
                        change the auth server URL
        --auth-client-id ID
                        change auth client id
        --auth-secret SECRET
                        change auth secret
        --auth-vin VIN  change auth vin
        --config PATH   change config path
        --ota-server URL
                        change ota server URL
        --ota-packages-dir PATH
                        change downloaded directory for packages
        --ota-package-manager MANAGER
                        change package manager

"#, bin_dir()));
}

#[test]
fn bad_auth_server_url() {
    assert_eq!(client(&["--auth-server", "apa"]),
               "Invalid auth-server URL: Url parse error: relative URL without a base\n")
}

#[test]
fn bad_ota_server_url() {
    assert_eq!(client(&["--ota-server", "apa"]),
               "Invalid ota-server URL: Url parse error: relative URL without a base\n")
}

#[test]
fn bad_section() {
    assert_eq!(client_with_config(&[""], "[uth]\n"),
               "parse_toml_table, invalid section: auth\n")
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

         // XXX:
         // "Failed to load config: Is a directory (os error 21)\n")
}
