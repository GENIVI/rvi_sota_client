use std::path::PathBuf;
use std::fmt;
use std::fs;

use time;
use rand;
use rand::Rng;
use log;
use log::{LogRecord, LogLevel, LogMetadata};

use message::PackageId;

macro_rules! test_init {
    () => {
        use test_library::SimpleLogger;
        use log::LogLevelFilter;
        use log;
        match log::set_logger(|max_log_level| {
            max_log_level.set(LogLevelFilter::Trace);
            Box::new(SimpleLogger)
        }) {
            Ok(..) => {},
            Err(..) => {}
        }
    }
}

pub struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= LogLevel::Info
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }
}

pub struct PathPrefix { prefix: String }

impl PathPrefix {
    pub fn new() -> PathPrefix {
        PathPrefix {
            prefix: format!("/tmp/rust-test-{}",
                            time::precise_time_ns()
                            .to_string())
        }
    }
    
    pub fn to_string(&self) -> String {
        return self.prefix.clone();
    }
}

impl Drop for PathPrefix {
    fn drop(&mut self) {
        let dir = PathBuf::from(&self.prefix);
        fs::remove_dir_all(dir).unwrap();
    }
}

impl fmt::Display for PathPrefix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.prefix)
    }
}

pub fn generate_random_package(i: usize) -> PackageId {
    PackageId {
        name: rand::thread_rng()
            .gen_ascii_chars().take(i).collect::<String>(),
        version: rand::thread_rng()
            .gen_ascii_chars().take(i).collect::<String>()
    }
}
