extern crate hyper;
extern crate rustc_serialize;
extern crate time;
extern crate url;
extern crate crypto;
extern crate toml;

#[macro_use] extern crate log;
extern crate env_logger;

#[cfg(test)] extern crate rand;

#[cfg(test)]
#[macro_use]
mod test_library;

/// Try to unwrap or log the error and run the second argument
/// Needs to run agains a expression, that returns a Option<T> and where E in
/// Error<E> implements Display
macro_rules! try_or {
    ($expr:expr, $finalize:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => {
                error!("{}", e);
                $finalize;
            }
        }
    }
}

/// Try to unwrap or log the provided message and run the third argument
macro_rules! try_msg_or {
    ($expr:expr, $msg:expr, $finalize:expr) => {
        match $expr {
            Ok(val) => val,
            Err(..) => {
                error!("{}", $msg);
                $finalize;
            }
        }
    }
}

pub mod rvi;
pub mod jsonrpc;

pub mod configuration;
pub mod handler;
pub mod message;
mod persistence;
