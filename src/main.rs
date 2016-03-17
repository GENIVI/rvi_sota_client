//! Logic for starting and configuring the sota_client from the command line.

extern crate sota_client;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate getopts;

use std::env;
use getopts::{Options, Matches};
use sota_client::configuration::Configuration;
use sota_client::genivi;

/// Helper function to print usage information to stdout.
///
/// # Arguments
/// * `program`: The invoking path or name of the executable
/// * `opts`: A pointer to a `Options` object, which generates the actual documentation. See the
///   [getopts documentation](https://doc.rust-lang.org/getopts/getopts/index.html) for details.
#[cfg_attr(test, allow(dead_code))]
fn print_usage(program: &str, opts: &Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

/// Parses the command line and matches it against accepted flags and options. Returns a `Matches`
/// object. See the [getopts documentation](https://doc.rust-lang.org/getopts/getopts/index.html)
/// for details.
///
/// # Arguments
/// * `args`: A pointer to a Array of Strings. This is supposed to be the commandline as returned
///    by [`env::args().collect()`](https://doc.rust-lang.org/stable/std/env/fn.args.html).
/// * `program`: The invoking path or name of the executable
fn match_args(args: &[String], program: &str) -> Matches {
    let mut options = Options::new();
    options.optflag("h", "help", "print this help message");
    options.optopt("c", "config", "change the path where the configuration \
                   is expected", "FILE");
    options.optopt("r", "rvi", "explicitly set the URL, where RVI can be \
                   reached", "URL");
    options.optopt("e", "edge", "explicitly set the host and port, where the \
                   client should listen for connections from RVI", "HOST:PORT");

    let matches = match options.parse(args) {
        Ok(m) => { m }
        Err(f) => {
            error!("{}", f.to_string());
            print_usage(program, &options);
            std::process::exit(1);
        }
    };

    if matches.opt_present("h") {
        print_usage(program, &options);
        std::process::exit(0);
    }
    matches
}

/// Program entrypoint. Parses command line arguments and starts the main loop accordingly.
#[cfg_attr(test, allow(dead_code))]
fn main() {
    env_logger::init().unwrap();
    let args: Vec<String> = env::args().collect();
    let program: &str = &args[0];
    let matches = match_args(&args[1..], program);

    let conf_file = matches.opt_str("c")
        .unwrap_or(Configuration::default_path());
    let configuration = match Configuration::read(&conf_file) {
        Ok(value) => value,
        Err(e) => {
            error!("Couldn't parse configuration file at {}: {}", conf_file, e);
            std::process::exit(126);
        }
    };

    let rvi_url: String = matches.opt_str("r")
        .unwrap_or(configuration.client.rvi_url.clone()
                   .unwrap_or("http://localhost:8901".to_string()));
    let edge_url: String = matches.opt_str("e")
        .unwrap_or(configuration.client.edge_url.clone()
                   .unwrap_or("localhost:9080".to_string()));

    genivi::start::start(&configuration, rvi_url, edge_url);
}
