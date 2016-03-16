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
               "Authentication error, Can\'t get AuthPlus token: Cannot send request: connection refused\n");
}
