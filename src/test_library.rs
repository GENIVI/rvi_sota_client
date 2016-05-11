//! Helper functions for testing `sota_client`.

use std::path::PathBuf;
use std::fmt;
use std::fs;

use time;
use log;
use log::{LogRecord, LogLevel, LogMetadata};

/// Initiates logging in tests. Can safely be called multiple times.
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

/// Implements a simple logger printing all log messages to stdout.
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

/// Wrapper for storing test data in a temporary directory. The created directory will be deleted,
/// when dropped.
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

