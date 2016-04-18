#[macro_use] extern crate log;
extern crate env_logger;
extern crate chan_signal;
extern crate chan;
extern crate getopts;
extern crate hyper;
extern crate ws;
extern crate rustc_serialize;
#[macro_use] extern crate libotaplus;

use getopts::Options;
use std::env;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread;
use std::time::Duration;
use chan_signal::Signal;
use chan::Receiver as ChanReceiver;

use libotaplus::auth_plus::authenticate;
use libotaplus::datatype::{config, Config, Event, Command, AccessToken, Url};
use libotaplus::http_client::HttpClient;
use libotaplus::interaction_library::broadcast::Broadcast;
use libotaplus::interaction_library::console::Console;
use libotaplus::interaction_library::gateway::Gateway;
use libotaplus::interaction_library::websocket::Websocket;
use libotaplus::interpreter::Interpreter;
use libotaplus::interaction_library::{Interpreter as InteractionInterpreter};
use libotaplus::package_manager::PackageManager;

macro_rules! spawn_thread {
    ($name:expr, $body:block) => {
        {
            match thread::Builder::new().name($name.to_string()).spawn(move || {
                info!("Spawning {}", $name.to_string());
                $body
            }) {
                Err(e) => panic!("Couldn't spawn {}: {}", $name, e),
                Ok(handle) => handle
            }
        }
    }
}

fn spawn_interpreter(config: Config, token: AccessToken, crx: Receiver<Command>, etx: Sender<Event>) {
    spawn_thread!("Interpreter", {
        Interpreter::<hyper::Client>::new(&config, token.clone(), crx, etx).start();
    });
}

fn spawn_autoacceptor(erx: Receiver<Event>, ctx: Sender<Command>) {
    spawn_thread!("Autoacceptor of software updates", {
        AutoAcceptor::run(&(), erx, ctx);
    });
}

fn spawn_signal_handler(signals: ChanReceiver<Signal>, ctx: Sender<Command>) {
    spawn_thread!("TERM signal handler", {
        loop {
            match signals.recv() {
                Some(s) if s == Signal::TERM => {
                    let _ = ctx.send(Command::Shutdown);
                },
                _ => {}
            }
        }
    });
}

fn spawn_update_poller(ctx: Sender<Command>, config: Config) {
    spawn_thread!("Update poller", {
        loop {
            let _ = ctx.send(Command::GetPendingUpdates);
            thread::sleep(Duration::from_secs(config.ota.polling_interval));
        }
    });
}

fn perform_initial_sync(ctx: Sender<Command>) {
    let _ = ctx.clone().send(Command::PostInstalledPackages);
}

fn start_event_broadcasting(broadcast: Broadcast<Event>) {
    spawn_thread!("Event Broadcasting", {
        broadcast.start();
    });
}

struct AutoAcceptor;

impl InteractionInterpreter<(), Event, Command> for AutoAcceptor {
    fn interpret(_: &(), e: Event, ctx: Sender<Command>) {
        fn f(e: &Event, ctx: Sender<Command>) {
            match e {
                &Event::NewUpdateAvailable(ref id) => {
                    let _ = ctx.send(Command::AcceptUpdate(id.clone()));
                },
                _ => {}
            }
        }

        match e {
            Event::Batch(ref evs) => {
                for ev in evs {
                    f(&ev, ctx.clone())
                }
            }
            e => f(&e, ctx)
        }
    }
}

fn main() {

    env_logger::init().expect("Couldn't initialize logger");

    let config = build_config();

    info!("Authenticating against AuthPlus...");
    let token = authenticate::<hyper::Client>(&config.auth).unwrap_or_else(|e| exit!("{}", e));
    let (etx, erx): (Sender<Event>, Receiver<Event>) = channel();
    let (ctx, crx): (Sender<Command>, Receiver<Command>) = channel();

    let mut broadcast: Broadcast<Event> = Broadcast::new(erx);

    // Must subscribe to the signal before spawning ANY other threads
    let signals = chan_signal::notify(&[Signal::TERM]);

    spawn_autoacceptor(broadcast.subscribe(), ctx.clone());
    spawn_interpreter(config.clone(), token.clone(), crx, etx);
    Websocket::run(ctx.clone(), broadcast.subscribe());
    spawn_update_poller(ctx.clone(), config.clone());

    let events_for_repl = broadcast.subscribe();

    start_event_broadcasting(broadcast);

    perform_initial_sync(ctx.clone());

    spawn_signal_handler(signals, ctx.clone());

    if config.test.looping {
        println!("Ota Plus Client REPL started.");
        Console::run(ctx.clone(), events_for_repl);
    } else {
        thread::sleep(Duration::from_secs(60000000));
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
    opts.optopt("", "auth-vin",
                "change auth vin", "VIN");
    opts.optopt("", "ota-server",
                "change ota server URL", "URL");
    opts.optopt("", "ota-packages-dir",
                "change downloaded directory for packages", "PATH");
    opts.optopt("", "ota-package-manager",
                "change package manager", "MANAGER");
    opts.optflag("", "repl",
                 "enable repl");

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

    if let Some(vin) = matches.opt_str("auth-vin") {
        config.auth.vin = vin;
    }

    if let Some(s) = matches.opt_str("ota-server") {
        match Url::parse(&s) {
            Ok(url)  => config.ota.server = url,
            Err(err) => exit!("Invalid ota-server URL: {}", err)
        }
    }

    if let Some(path) = matches.opt_str("ota-packages-dir") {
        config.ota.packages_dir = path;
    }

    if let Some(s) = matches.opt_str("ota-package-manager") {
        config.ota.package_manager = match s.to_lowercase().as_str() {
            "dpkg" => PackageManager::Dpkg,
            "rpm"  => PackageManager::Rpm,
            path   => PackageManager::File(path.to_string()),
        }
    }

    if matches.opt_present("repl") {
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
