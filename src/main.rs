extern crate chan;
extern crate chan_signal;
extern crate crossbeam;
extern crate env_logger;
extern crate getopts;
extern crate hyper;
#[macro_use] extern crate log;
extern crate rustc_serialize;
extern crate time;
extern crate ws;
#[macro_use] extern crate libotaplus;

use chan::Receiver as ChanReceiver;
use chan_signal::Signal;
use env_logger::LogBuilder;
use getopts::Options;
use log::LogRecord;
use std::env;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread;
use std::time::Duration;

use libotaplus::datatype::{config, Command, Config, Event, Url};
use libotaplus::interaction_library::{Console, Gateway, Http, Websocket};
use libotaplus::interaction_library::broadcast::Broadcast;
use libotaplus::interaction_library::gateway::Interpret;
use libotaplus::interpreter::{EventInterpreter, Interpreter, Wrapped, WrappedInterpreter};
use libotaplus::package_manager::PackageManager;


fn spawn_signal_handler(signals: ChanReceiver<Signal>, ctx: Sender<Command>) {
    loop {
        match signals.recv() {
            Some(Signal::TERM) | Some(Signal::INT) => {
                ctx.send(Command::Shutdown).expect("send failed.")
            }
            _ => {}
        }
    }
}

fn spawn_update_poller(ctx: Sender<Command>, config: Config) {
    loop {
        let _ = ctx.send(Command::GetPendingUpdates);
        thread::sleep(Duration::from_secs(config.ota.polling_interval))
    }
}

fn spawn_command_forwarder(crx: Receiver<Command>, wtx: Sender<Wrapped>) {
    loop {
        match crx.recv() {
            Ok(cmd)  => wtx.send(Interpret { cmd: cmd, etx: None }).unwrap(),
            Err(err) => error!("Error receiving command to forward: {:?}", err),
        }
    }
}

fn perform_initial_sync(ctx: Sender<Command>) {
    let _ = ctx.clone().send(Command::Authenticate(None));
    let _ = ctx.clone().send(Command::UpdateInstalledPackages);
}

fn main() {
    setup_logging();
    let config = build_config();

    let (ctx, crx): (Sender<Command>, Receiver<Command>) = channel();
    let (etx, erx): (Sender<Event>,   Receiver<Event>)   = channel();
    let (wtx, wrx): (Sender<Wrapped>, Receiver<Wrapped>) = channel();
    let mut broadcast: Broadcast<Event> = Broadcast::new(erx);

    crossbeam::scope(|scope| {
        // Must subscribe to the signal before spawning ANY other threads
        let signals = chan_signal::notify(&[Signal::INT, Signal::TERM]);
        let sig_ctx = ctx.clone();
        scope.spawn(move || spawn_signal_handler(signals, sig_ctx));

        let sync_ctx = ctx.clone();
        scope.spawn(move || perform_initial_sync(sync_ctx));

        let poll_ctx = ctx.clone();
        let poll_cfg = config.clone();
        scope.spawn(move || spawn_update_poller(poll_ctx, poll_cfg));

        let cmd_wtx = wtx.clone();
        scope.spawn(move || spawn_command_forwarder(crx, cmd_wtx));

        let ev_sub = broadcast.subscribe();
        let ev_ctx = ctx.clone();
        let mut ev_int = EventInterpreter;
        scope.spawn(move || ev_int.run(ev_sub, ev_ctx));

        let mut w_int = WrappedInterpreter { config: config.clone(), access_token: None };
        scope.spawn(move || w_int.run(wrx, etx));

        let ws_wtx = wtx.clone();
        let ws_sub = broadcast.subscribe();
        scope.spawn(move || Websocket::run(ws_wtx, ws_sub));

        if config.test.http {
            let http_wtx = wtx.clone();
            let http_sub = broadcast.subscribe();
            scope.spawn(move || Http::run(http_wtx, http_sub));
        }

        if config.test.looping {
            println!("OTA Plus Client REPL started.");
            let cons_wtx = wtx.clone();
            let cons_sub = broadcast.subscribe();
            scope.spawn(move || Console::run(cons_wtx, cons_sub));
        }

        scope.spawn(move || broadcast.start());
    });
}

fn setup_logging() {
    let format = |record: &LogRecord| {
        let service_name = env::var("SERVICE_NAME")
            .unwrap_or("ota-plus-client".to_string());

        let service_version = env::var("SERVICE_VERSION")
            .unwrap_or("?".to_string());

        let timestamp = format!("{}", time::now().ctime());

        format!("{} ({}), {}: {} - {}",
                service_name, service_version, timestamp, record.level(), record.args())
    };

    let mut builder = LogBuilder::new();
    builder.format(format);

    if let Ok(level) = env::var("RUST_LOG") {
        builder.parse(&level);
    }

    builder.init().expect("env_logger::init() called twice, blame the programmers.");
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
    opts.optflag("", "http",
                 "enable interaction via http requests");

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
            path   => PackageManager::File { filename: path.to_string(), succeeds: true },
        }
    }

    if matches.opt_present("repl") {
        config.test.looping = true;
    }

    if matches.opt_present("http") {
        config.test.http = true;
    }

    config
}
