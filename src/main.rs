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


fn spawn_signal_handler(signals: Receiver<Signal>) {
    loop {
        match signals.recv() {
            Some(Signal::INT)  => std::process::exit(0),
            Some(Signal::TERM) => std::process::exit(0),
            _                  => ()
        }
    }
}

fn spawn_update_poller(interval: u64, ctx: Sender<Command>) {
    let tick = chan::tick(Duration::from_secs(interval));
    loop {
        let _ = tick.recv();
        ctx.send(Command::GetPendingUpdates);
    }
}

fn perform_initial_sync(ctx: &Sender<Command>) {
    ctx.send(Command::Authenticate(None));
    ctx.send(Command::UpdateInstalledPackages);
}

use std::sync::{Arc, Mutex};
use libotaplus::datatype::config::ClientConfiguration;
use libotaplus::remote::svc::{RemoteServices, ServiceHandler};
use libotaplus::remote::rvi::{ServiceEdge};
use libotaplus::swm::interpreter::{UpstreamInterpreter, SwmEventInterpreter};

fn build_remote(tx: Sender<Event>) -> Arc<Mutex<RemoteServices>> {
    let cfg = ClientConfiguration::default();

    let remote_svcs = Arc::new(Mutex::new(RemoteServices::new(cfg.rvi_url.inner().clone())));
    let handler = ServiceHandler::new(tx, remote_svcs.clone(), cfg.clone());
    let rvi_edge = ServiceEdge::new(cfg.rvi_url.inner().clone(), cfg.edge_url.inner(), handler);
    rvi_edge.start();
    remote_svcs
}

fn main() {
    setup_logging();
    let config = build_config();

    let (etx, erx) = chan::async::<Event>();
    let (ctx, crx) = chan::async::<Command>();
    let (gtx, grx) = chan::async::<Global>();

    let mut broadcast = Broadcast::new(erx);
    perform_initial_sync(&ctx);

    crossbeam::scope(|scope| {
        // Must subscribe to the signal before spawning ANY other threads
        let signals = chan_signal::notify(&[Signal::INT, Signal::TERM]);
        scope.spawn(move || spawn_signal_handler(signals));

        let poll_tick = config.ota.polling_interval;
        let poll_ctx  = ctx.clone();
        scope.spawn(move || spawn_update_poller(poll_tick, poll_ctx));

        if config.test.http {
            let http_gtx = gtx.clone();
            let http_sub = broadcast.subscribe();
            scope.spawn(move || Http::run(http_gtx, http_sub));
        }

        if config.test.repl {
            let repl_gtx = gtx.clone();
            let repl_sub = broadcast.subscribe();
            scope.spawn(move || Console::run(repl_gtx, repl_sub));
        }

        if config.test.websocket {
            let ws_gtx = gtx.clone();
            let ws_sub = broadcast.subscribe();
            scope.spawn(move || Websocket::run(ws_gtx, ws_sub));
        }

        // TODO: Enable RVI and SWLM DBus
        if true {
            let event_sub = broadcast.subscribe();
            let event_ctx = ctx.clone();
            scope.spawn(move || EventInterpreter.run(event_sub, event_ctx));

            let cmd_gtx = gtx.clone();
            scope.spawn(move || CommandInterpreter.run(crx, cmd_gtx));

            scope.spawn(move || GlobalInterpreter {
                config:      config,
                token:       None,
                http_client: Box::new(AuthClient::new(Auth::None)),
                loopback_tx: gtx,
            }.run(grx, etx));
        } else {
            let remote_svcs = build_remote(etx.clone());

            let ev_sub = broadcast.subscribe();
            let ev_ctx = ctx.clone();
            let mut ev_int = SwmEventInterpreter;
            scope.spawn(move || ev_int.run(ev_sub, ev_ctx));

            let mut w_int = UpstreamInterpreter { upstream: remote_svcs };
            scope.spawn(move || w_int.run(grx, etx.clone()));
        }

        scope.spawn(move || broadcast.start());
    });
}

fn setup_logging() {
    let name    = option_env!("SERVICE_NAME").unwrap_or("ota-plus-client");
    let version = option_env!("SERVICE_VERSION").unwrap_or("?");

    let mut builder = LogBuilder::new();
    builder.format(move |record: &LogRecord| {
        let timestamp = format!("{}", time::now_utc().rfc3339());
        format!("{}:{} @ {}: {} - {}", name, version, timestamp, record.level(), record.args())
    });
    let _ = env::var("RUST_LOG").map(|level| builder.parse(&level));
    builder.init().expect("env_logger::init() called twice, blame the programmers.");
}


fn build_config() -> Config {
    let args     = env::args().collect::<Vec<String>>();
    let program  = args[0].clone();
    let mut opts = Options::new();

    opts.optflag("h", "help", "print this help menu");
    opts.optflag("", "repl", "enable repl");
    opts.optflag("", "http", "enable interaction via http requests");
    opts.optflag("", "no-websocket", "disable websocket interaction");

    opts.optopt("", "auth-server", "change the auth server URL", "URL");
    opts.optopt("", "auth-client-id", "change auth client id", "ID");
    opts.optopt("", "auth-secret", "change auth secret", "SECRET");
    opts.optopt("", "auth-vin", "change auth vin", "VIN");
    opts.optopt("", "config", "change config path", "PATH");
    opts.optopt("", "ota-server", "change ota server URL", "URL");
    opts.optopt("", "ota-packages-dir", "change downloaded directory for packages", "PATH");
    opts.optopt("", "ota-package-manager", "change package manager", "MANAGER");

    let matches     = opts.parse(&args[1..]).unwrap_or_else(|err| panic!(err.to_string()));
    let config_file = matches.opt_str("config").unwrap_or_else(|| {
        env::var("OTA_PLUS_CLIENT_CFG").unwrap_or("/opt/ats/ota/etc/ota.toml".to_string())
    });
    let mut config  = config::load_config(&config_file).unwrap_or_else(|err| exit!("{}", err));

    if matches.opt_present("h") {
        exit!("{}", opts.usage(&format!("Usage: {} [options]", program)));
    }
    if matches.opt_present("repl") {
        config.test.repl = true;
    }
    if matches.opt_present("http") {
        config.test.http = true;
    }
    if matches.opt_present("no-websocket") {
        config.test.websocket = false;
    }

    matches.opt_str("auth-client-id").map(|id| config.auth.client_id = id);
    matches.opt_str("auth-secret").map(|secret| config.auth.secret = secret);
    matches.opt_str("auth-vin").map(|vin| config.auth.vin = vin);
    matches.opt_str("ota-packages-dir").map(|path| config.ota.packages_dir = path);

    matches.opt_str("auth-server").map(|text| {
        config.auth.server = Url::parse(&text).unwrap_or_else(|err| exit!("Invalid auth-server URL: {}", err));
    });
    matches.opt_str("ota-server").map(|text| {
        config.ota.server  = Url::parse(&text).unwrap_or_else(|err| exit!("Invalid ota-server URL: {}", err));
    });
    matches.opt_str("ota-package-manager").map(|text| {
        config.ota.package_manager = text.parse::<PackageManager>().unwrap_or_else(|err| exit!("Invalid package manager: {}", err));
    });

    config
}
