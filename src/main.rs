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
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use libotaplus::datatype::{config, Command, Config, Event};
use libotaplus::gateway::{Console, DBus, Gateway, Interpret, Http, Websocket};
use libotaplus::gateway::broadcast::Broadcast;
use libotaplus::http::AuthClient;
use libotaplus::interpreter::{EventInterpreter, CommandInterpreter, Interpreter, GlobalInterpreter};
use libotaplus::rvi::{Edge, Services};


fn start_signal_handler(signals: Receiver<Signal>) {
    loop {
        match signals.recv() {
            Some(Signal::INT)  => std::process::exit(0),
            Some(Signal::TERM) => std::process::exit(0),
            _                  => ()
        }
    }
}

fn start_update_poller(interval: u64, ctx: Sender<Command>) {
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

fn main() {
    setup_logging();
    let config = build_config();

    let (etx, erx) = chan::async::<Event>();
    let (ctx, crx) = chan::async::<Command>();
    let (itx, irx) = chan::async::<Interpret>();

    let mut broadcast = Broadcast::new(erx);
    perform_initial_sync(&ctx);

    crossbeam::scope(|scope| {
        // Must subscribe to the signal before spawning ANY other threads
        let signals = chan_signal::notify(&[Signal::INT, Signal::TERM]);
        scope.spawn(move || start_signal_handler(signals));

        let poll_tick = config.device.polling_interval;
        let poll_ctx  = ctx.clone();
        scope.spawn(move || start_update_poller(poll_tick, poll_ctx));

        if config.gateway.console {
            let cons_itx = itx.clone();
            let cons_sub = broadcast.subscribe();
            scope.spawn(move || Console.start(cons_itx, cons_sub));
        }

        let mut rvi = None;
        if config.gateway.dbus {
            let services = Services::new(config.rvi.clone(), config.device.uuid.clone(), etx.clone());
            let mut edge = Edge::new(services.clone(), config.rvi.edge.clone(), config.rvi.client.clone());
            scope.spawn(move || edge.start());
            rvi = Some(services);

            let dbus_itx = itx.clone();
            let dbus_sub = broadcast.subscribe();
            let mut dbus = DBus { dbus_cfg: config.dbus.clone(), itx: itx.clone() };
            scope.spawn(move || dbus.start(dbus_itx, dbus_sub));
        }

        if config.gateway.http {
            let http_itx = itx.clone();
            let http_sub = broadcast.subscribe();
            scope.spawn(move || Http.start(http_itx, http_sub));
        }

        if config.gateway.websocket {
            let ws_itx = itx.clone();
            let ws_sub = broadcast.subscribe();
            let mut ws = Websocket { clients: Arc::new(Mutex::new(HashMap::new())) };
            scope.spawn(move || ws.start(ws_itx, ws_sub));
        }

        let event_sub = broadcast.subscribe();
        let event_ctx = ctx.clone();
        scope.spawn(move || EventInterpreter.run(event_sub, event_ctx));

        let cmd_itx = itx.clone();
        scope.spawn(move || CommandInterpreter.run(crx, cmd_itx));

        scope.spawn(move || GlobalInterpreter {
            config:      config,
            token:       None,
            http_client: Box::new(AuthClient::new()),
            rvi:         rvi,
            loopback_tx: itx,
        }.run(irx, etx));

        scope.spawn(move || broadcast.start());
    });
}

fn setup_logging() {
    let name    = option_env!("SERVICE_NAME").unwrap_or("sota_client");
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
    opts.optopt("", "config", "change config path", "PATH");

    opts.optopt("", "auth-server", "change the auth server", "URL");
    opts.optopt("", "auth-client-id", "change the auth client id", "ID");
    opts.optopt("", "auth-secret", "change the auth secret", "SECRET");
    opts.optopt("", "auth-credentials-file", "change the auth credentials file", "PATH");

    opts.optopt("", "core-server", "change the core server", "URL");

    opts.optopt("", "dbus-name", "change the dbus registration name", "NAME");
    opts.optopt("", "dbus-path", "change the dbus path", "PATH");
    opts.optopt("", "dbus-interface", "change the dbus interface name", "INTERFACE");
    opts.optopt("", "dbus-software-manager", "change the dbus software manager name", "NAME");
    opts.optopt("", "dbus-software-manager-path", "change the dbus software manager path", "PATH");
    opts.optopt("", "dbus-timeout", "change the dbus installation timeout", "TIMEOUT");

    opts.optopt("", "device-uuid", "change the device uuid", "UUID");
    opts.optopt("", "device-vin", "change the device vin", "VIN");
    opts.optopt("", "device-packages-dir", "change downloaded directory for packages", "PATH");
    opts.optopt("", "device-package-manager", "change the package manager", "MANAGER");
    opts.optopt("", "device-polling-interval", "change the package polling interval", "INTERVAL");

    opts.optopt("", "gateway-console", "toggle the console gateway", "BOOL");
    opts.optopt("", "gateway-dbus", "toggle the dbus gateway", "BOOL");
    opts.optopt("", "gateway-http", "toggle the http gateway", "BOOL");
    opts.optopt("", "gateway-websocket", "toggle the websocket gateway", "BOOL");

    opts.optopt("", "rvi-client", "change the rvi client URL", "URL");
    opts.optopt("", "rvi-edge", "change the exposed rvi edge URL", "URL");
    opts.optopt("", "rvi-storage-dir", "change the rvi storage directory", "PATH");
    opts.optopt("", "rvi-timeout", "change the rvi timeout", "TIMEOUT");

    let matches = opts.parse(&args[1..]).unwrap_or_else(|err| panic!(err.to_string()));
    if matches.opt_present("h") {
        exit!("{}", opts.usage(&format!("Usage: {} [options]", program)));
    }

    let config_file = matches.opt_str("config").unwrap_or_else(|| {
        env::var("SOTA_CONFIG").unwrap_or("/etc/sota.toml".to_string())
    });
    let mut config  = config::load_config(&config_file).unwrap_or_else(|err| exit!("{}", err));

    config.auth.as_mut().map(|auth_cfg| {
        matches.opt_str("auth-client-id").map(|id| auth_cfg.client_id = id);
        matches.opt_str("auth-secret").map(|secret| auth_cfg.secret = secret);
        matches.opt_str("auth-server").map(|text| {
            auth_cfg.server = text.parse().unwrap_or_else(|err| exit!("Invalid auth-server URL: {}", err));
        });
    });

    matches.opt_str("core-server").map(|text| {
        config.core.server = text.parse().unwrap_or_else(|err| exit!("Invalid core-server URL: {}", err));
    });

    matches.opt_str("dbus-name").map(|name| config.dbus.name = name);
    matches.opt_str("dbus-path").map(|path| config.dbus.path = path);
    matches.opt_str("dbus-interface").map(|interface| config.dbus.interface = interface);
    matches.opt_str("dbus-software-manager").map(|mgr| config.dbus.software_manager = mgr);
    matches.opt_str("dbus-software-manager-path").map(|mgr_path| config.dbus.software_manager_path = mgr_path);
    matches.opt_str("dbus-timeout").map(|timeout| {
        config.dbus.timeout = timeout.parse().unwrap_or_else(|err| exit!("Invalid dbus timeout: {}", err));
    });

    matches.opt_str("device-uuid").map(|uuid| config.device.uuid = uuid);
    matches.opt_str("device-vin").map(|vin| config.device.vin = vin);
    matches.opt_str("device-packages-dir").map(|path| config.device.packages_dir = path);
    matches.opt_str("device-package-manager").map(|text| {
        config.device.package_manager = text.parse().unwrap_or_else(|err| exit!("Invalid device package manager: {}", err));
    });
    matches.opt_str("device-polling-interval").map(|interval| {
        config.device.polling_interval = interval.parse().unwrap_or_else(|err| exit!("Invalid device polling interval: {}", err));
    });

    matches.opt_str("gateway-console").map(|console| {
        config.gateway.console = console.parse().unwrap_or_else(|err| exit!("Invalid console gateway boolean: {}", err));
    });
    matches.opt_str("gateway-dbus").map(|dbus| {
        config.gateway.dbus = dbus.parse().unwrap_or_else(|err| exit!("Invalid dbus gateway boolean: {}", err));
    });
    matches.opt_str("gateway-http").map(|http| {
        config.gateway.http = http.parse().unwrap_or_else(|err| exit!("Invalid http gateway boolean: {}", err));
    });
    matches.opt_str("gateway-websocket").map(|websocket| {
        config.gateway.websocket = websocket.parse().unwrap_or_else(|err| exit!("Invalid websocket gateway boolean: {}", err));
    });

    matches.opt_str("rvi-client").map(|url| {
        config.rvi.client = url.parse().unwrap_or_else(|err| exit!("Invalid rvi-client URL: {}", err));
    });
    matches.opt_str("rvi-edge").map(|url| {
        config.rvi.edge = url.parse().unwrap_or_else(|err| exit!("Invalid rvi-edge: {}", err));
    });
    matches.opt_str("rvi-storage-dir").map(|dir| config.rvi.storage_dir = dir);
    matches.opt_str("rvi-timeout").map(|timeout| {
        config.rvi.timeout = Some(timeout.parse().unwrap_or_else(|err| exit!("Invalid rvi timeout: {}", err)));
    });

    config
}
