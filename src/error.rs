use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io;

#[derive(Debug)]
pub enum Error {
    AuthError(String),
    Ota(OtaReason),
    ParseError(String),
    PackageError(String),
    ClientError(String),
    Config(ConfigReason),
    Io(io::Error)
}

use std::convert::From;

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

#[derive(Debug)]
pub enum OtaReason {
    CreateFile(io::Error),
}

#[derive(Debug)]
pub enum ConfigReason {
    Parse(ParseReason),
    Io(io::Error),
}

#[derive(Debug)]
pub enum ParseReason {
    InvalidToml,
    InvalidSection(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let inner: String = match *self {
            Error::AuthError(ref s)    => format!("Authentication error, {}", s.clone()),
            Error::Ota(ref e)          => format!("Ota server error, {}", e.clone()),
            Error::ParseError(ref s)   => s.clone(),
            Error::PackageError(ref s) => s.clone(),
            Error::ClientError(ref s)  => s.clone(),
            Error::Config(ref e)       => format!("Failed to {}", e.clone()),
            Error::Io(ref e)           => format!("IO Error{:?}", e.clone()),
        };
        write!(f, "{}", inner)
    }
}

impl Display for OtaReason {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let inner: String = match *self {
            OtaReason::CreateFile(ref e) => format!("failed to create file: {}", e.clone())
        };
        write!(f, "{}", inner)
    }
}

impl Display for ConfigReason {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let inner: String = match *self {
            ConfigReason::Parse(ref e) => format!("parse config: {}", e.clone()),
            ConfigReason::Io   (ref e) => format!("load config: {}", e.clone())
        };
        write!(f, "{}", inner)
    }
}


impl Display for ParseReason {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let inner: String = match *self {
            ParseReason::InvalidToml           => "invalid toml".to_string(),
            ParseReason::InvalidSection(ref s) => format!("invalid section: {}", s),
        };
        write!(f, "{}", inner)
    }
}

#[macro_export]
macro_rules! exit {
    ($fmt:expr) => ({
        print!(concat!($fmt, "\n"));
        std::process::exit(1);
    });
    ($fmt:expr, $($arg:tt)*) => ({
        print!(concat!($fmt, "\n"), $($arg)*);
        std::process::exit(1);
    })
}
