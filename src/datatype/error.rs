use std::convert::From;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::Error as IoError;
use std::path::PathBuf;
use std::string::FromUtf8Error;
use std::sync::PoisonError;
use std::sync::mpsc::SendError;
use url::ParseError as UrlParseError;

use datatype::Event;
use rustc_serialize::json::{EncoderError as JsonEncoderError, DecoderError as JsonDecoderError};
use hyper::error::Error as HyperError;
use ws::Error as WebsocketError;


#[derive(Debug)]
pub enum Error {
    AuthError(String),
    ClientError(String),
    Command(String),
    Config(ConfigReason),
    FromUtf8Error(FromUtf8Error),
    HyperError(HyperError),
    IoError(IoError),
    JsonDecoderError(JsonDecoderError),
    JsonEncoderError(JsonEncoderError),
    PoisonError(String),
    Ota(OtaReason),
    PackageError(String),
    ParseError(String),
    SendErrorEvent(SendError<Event>),
    UrlParseError(UrlParseError),
    WebsocketError(WebsocketError),
}

macro_rules! derive_from {
    ([ $( $error: ident ),* ]) =>
    {
        $(
            impl From<$error> for Error {
                fn from(e: $error) -> Error {
                    Error::$error(e)
                }
            }
        )*
    }
}

derive_from!(
    [ JsonEncoderError
    , JsonDecoderError
    , HyperError
    , FromUtf8Error
    , IoError
    , UrlParseError
    , WebsocketError
    ]);

impl From<SendError<Event>> for Error {
    fn from(e: SendError<Event>) -> Error {
        Error::SendErrorEvent(e)
    }
}

impl<E> From<PoisonError<E>> for Error {
    fn from(e: PoisonError<E>) -> Error {
        Error::PoisonError(format!("{}", e))
    }
}

#[derive(Debug)]
pub enum OtaReason {
    CreateFile(PathBuf, IoError),
    Client(String, String),
}

#[derive(Debug)]
pub enum ConfigReason {
    Parse(ParseReason),
    Io(IoError),
}

#[derive(Debug)]
pub enum ParseReason {
    InvalidToml,
    InvalidSection(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let inner: String = match *self {
            Error::AuthError(ref s)        => format!("Authentication error, {}", s.clone()),
            Error::ClientError(ref s)      => format!("Http client error: {}", s.clone()),
            Error::Command(ref e)          => format!("Unknown Command: {}", e.clone()),
            Error::Config(ref e)           => format!("Failed to {}", e.clone()),
            Error::FromUtf8Error(ref e)    => format!("From utf8 error: {}", e.clone()),
            Error::HyperError(ref e)       => format!("Hyper error: {}", e.clone()),
            Error::IoError(ref e)          => format!("IO error: {}", e.clone()),
            Error::JsonDecoderError(ref e) => format!("Failed to decode JSON: {}", e.clone()),
            Error::JsonEncoderError(ref e) => format!("Failed to encode JSON: {}", e.clone()),
            Error::Ota(ref e)              => format!("Ota error, {}", e.clone()),
            Error::PoisonError(ref e)      => format!("Poison error, {}", e.clone()),
            Error::PackageError(ref s)     => s.clone(),
            Error::ParseError(ref s)       => s.clone(),
            Error::SendErrorEvent(ref s)   => format!("Send error for Event: {}", s.clone()),
            Error::UrlParseError(ref s)    => format!("Url parse error: {}", s.clone()),
            Error::WebsocketError(ref e)   => format!("Websocket Error{:?}", e.clone()),
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
