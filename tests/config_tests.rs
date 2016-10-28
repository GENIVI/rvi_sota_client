use std::env;
use std::path::Path;
use std::process::{Command, Output};


fn run_client(config: &str) -> Output {
    let out_dir = env::var("OUT_DIR").expect("expected OUT_DIR environment variable");
    let bin_dir = Path::new(&out_dir).parent().unwrap().parent().unwrap().parent().unwrap();

    Command::new(format!("{}/sota_client", bin_dir.to_str().unwrap()))
        .arg("--print")
        .arg(format!("--config={}", config))
        .output()
        .unwrap_or_else(|err| panic!("couldn't start client: {}", err))
}


#[test]
fn default_config() {
    assert!(run_client("tests/toml/default.toml").status.success());
}

#[test]
fn genivi_config() {
    assert!(run_client("tests/toml/genivi.toml").status.success());
}

#[test]
fn old_config() {
    assert!(run_client("tests/toml/old.toml").status.success());
}

#[test]
fn polling_config() {
    assert!(run_client("tests/toml/polling.toml").status.success() != true);
}
