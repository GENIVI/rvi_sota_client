#[macro_use] extern crate chan;
extern crate chan_signal;
extern crate crossbeam;
extern crate env_logger;
extern crate getopts;
extern crate hyper;
#[macro_use] extern crate libotaplus;
#[macro_use] extern crate log;
extern crate rustc_serialize;
extern crate time;
extern crate ws;

use chan::{Sender, Receiver};
use chan_signal::Signal;
use env_logger::LogBuilder;
use getopts::Options;
use log::LogRecord;
use std::env;
use std::time::Duration;

use libotaplus::datatype::{config, Auth, Command, Config, Event, Url};
use libotaplus::http_client::AuthClient;
use libotaplus::interaction_library::{Console, Gateway, Http, Websocket};
use libotaplus::interaction_library::broadcast::Broadcast;
use libotaplus::interpreter::{EventInterpreter, CommandInterpreter, Interpreter,
                              Global, GlobalInterpreter};
use libotaplus::package_manager::PackageManager;


fn spawn_signal_handler(signals: Receiver<Signal>, ctx: Sender<Command>, shutdown_rx: Receiver<()>) {
    loop {
        chan_select! {
            shutdown_rx.recv()    => std::process::exit(0),
            signals.recv() -> sig => match sig {
                Some(Signal::INT)  => ctx.send(Command::Shutdown),
                Some(Signal::TERM) => ctx.send(Command::Shutdown),
                _                  => ()
            },
        }
    }
}

fn spawn_update_poller(interval: u64, ctx: Sender<Command>, shutdown_rx: Receiver<()>) {
    let tick = chan::tick(Duration::from_secs(interval));
    loop {
        chan_select! {
            shutdown_rx.recv() => break,
            tick.recv()        => ctx.send(Command::GetPendingUpdates),
        }
    }
}

fn perform_initial_sync(ctx: &Sender<Command>) {
    ctx.send(Command::Authenticate(None));
    ctx.send(Command::UpdateInstalledPackages);
}

fn main() {
    setup_logging();
    let config = build_config();

    let (etx, erx) = chan::async::<Event>();
    let (ctx, crx) = chan::async::<Command>();
    let (gtx, grx) = chan::async::<Global>();
    let (shutdown_tx, shutdown_rx) = chan::sync::<()>(0);

    let mut broadcast = Broadcast::new(erx);
    let mut shutdown  = Broadcast::new(shutdown_rx);

    perform_initial_sync(&ctx);

    crossbeam::scope(|scope| {
        // Must subscribe to the signal before spawning ANY other threads
        let signals         = chan_signal::notify(&[Signal::INT, Signal::TERM]);
        let signal_ctx      = ctx.clone();
        let signal_shutdown = shutdown.subscribe();
        scope.spawn(move || spawn_signal_handler(signals, signal_ctx, signal_shutdown));

        let ws_gtx      = gtx.clone();
        let ws_event    = broadcast.subscribe();
        let ws_shutdown = shutdown.subscribe();
        scope.spawn(move || Websocket::run(ws_gtx, ws_event, ws_shutdown));

        if config.test.http {
            let http_gtx      = gtx.clone();
            let http_event    = broadcast.subscribe();
            let http_shutdown = shutdown.subscribe();
            scope.spawn(move || Http::run(http_gtx, http_event, http_shutdown));
        }

        if config.test.looping {
            let repl_gtx      = gtx.clone();
            let repl_event    = broadcast.subscribe();
            let repl_shutdown = shutdown.subscribe();
            scope.spawn(move || Console::run(repl_gtx, repl_event, repl_shutdown));
        }

        let event_subscribe = broadcast.subscribe();
        let event_ctx       = ctx.clone();
        let event_shutdown  = shutdown.subscribe();
        scope.spawn(move || EventInterpreter.run(event_subscribe, event_ctx, event_shutdown));

        let cmd_gtx      = gtx.clone();
        let cmd_shutdown = shutdown.subscribe();
        scope.spawn(move || CommandInterpreter.run(crx, cmd_gtx, cmd_shutdown));

        let poll_interval   = config.ota.polling_interval;
        let global_shutdown = shutdown.subscribe();
        scope.spawn(move || GlobalInterpreter {
            config:      config,
            token:       None,
            http_client: Box::new(AuthClient::new(Auth::None)),
            loopback_tx: gtx,
            shutdown_tx: shutdown_tx,
        }.run(grx, etx, global_shutdown));

        let poll_ctx      = ctx.clone();
        let poll_shutdown = shutdown.subscribe();
        scope.spawn(move || spawn_update_poller(poll_interval, poll_ctx, poll_shutdown));

        scope.spawn(move || shutdown.start());
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
