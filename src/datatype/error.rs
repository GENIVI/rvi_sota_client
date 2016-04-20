use hyper::error as hyper;
use rustc_serialize::json;
use std::convert::From;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io;
use std::path::PathBuf;
use url::ParseError as UrlParseError;
use ws;


#[derive(Debug)]
pub enum Error {
    AuthError(String),
    ClientError(String),
    Config(ConfigReason),
    Hyper(hyper::Error),
    Io(io::Error),
    JsonDecode(json::DecoderError),
    JsonEncode(json::EncoderError),
    Ota(OtaReason),
    PackageError(String),
    ParseError(String),
    UrlParseError(UrlParseError),
    Websocket(ws::Error),
}

impl From<json::EncoderError> for Error {
    fn from(e: json::EncoderError) -> Error {
        Error::JsonEncode(e)
    }
}

impl From<hyper::Error> for Error {
    fn from(e: hyper::Error) -> Error {
        Error::Hyper(e)
    }
}

impl From<json::DecoderError> for Error {
    fn from(e: json::DecoderError) -> Error {
        Error::JsonDecode(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

impl From<UrlParseError> for Error {
    fn from(e: UrlParseError) -> Error {
        Error::UrlParseError(e)
    }
}

impl From<ws::Error> for Error {
    fn from(e: ws::Error) -> Error {
        Error::Websocket(e)
    }
}

#[derive(Debug)]
pub enum OtaReason {
    CreateFile(PathBuf, io::Error),
    Client(String, String),
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
            Error::AuthError(ref s)     => format!("Authentication error, {}", s.clone()),
            Error::ClientError(ref s)   => s.clone(),
            Error::Config(ref e)        => format!("Failed to {}", e.clone()),
            Error::Hyper(ref e)         => format!("Hyper error: {}", e.clone()),
            Error::Io(ref e)            => format!("IO Error{:?}", e.clone()),
            Error::JsonDecode(ref e)    => format!("Failed to decode JSON: {}", e.clone()),
            Error::JsonEncode(ref e)    => format!("Failed to encode JSON: {}", e.clone()),
            Error::Ota(ref e)           => format!("Ota error, {}", e.clone()),
            Error::PackageError(ref s)  => s.clone(),
            Error::ParseError(ref s)    => s.clone(),
            Error::UrlParseError(ref s) => format!("Url parse error: {}", s.clone()),
            Error::Websocket(ref e)     => format!("Websocket Error{:?}", e.clone()),
        };
        write!(f, "{}", inner)
    }
}

impl Display for OtaReason {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let inner: String = match *self {
            OtaReason::CreateFile(ref f, ref e) =>
                format!("failed to create file {:?}: {}", f.clone(), e.clone()),
            OtaReason::Client(ref r, ref e) =>
                format!("the request: {},\nresults in the following error: {}", r.clone(), e.clone()),
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
