use std::convert::From;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::Error as IoError;
use std::string::FromUtf8Error;
use std::sync::PoisonError;
use std::sync::mpsc::SendError;
use toml::{ParserError as TomlParserError, DecodeError as TomlDecodeError};
use url::ParseError as UrlParseError;

use datatype::Event;
use rustc_serialize::json::{EncoderError as JsonEncoderError, DecoderError as JsonDecoderError};
use hyper::error::Error as HyperError;
use ws::Error as WebsocketError;


#[derive(Debug)]
pub enum Error {
    ClientError(String),
    Command(String),
    FromUtf8Error(FromUtf8Error),
    HyperError(HyperError),
    IoError(IoError),
    JsonDecoderError(JsonDecoderError),
    JsonEncoderError(JsonEncoderError),
    PoisonError(String),
    PackageError(String),
    ParseError(String),
    SendErrorEvent(SendError<Event>),
    TomlParserErrors(Vec<TomlParserError>),
    TomlDecodeError(TomlDecodeError),
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
    , TomlDecodeError
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

impl From<Vec<TomlParserError>> for Error {
    fn from(e: Vec<TomlParserError>) -> Error {
        Error::TomlParserErrors(e)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let inner: String = match *self {
            Error::ClientError(ref s)      => format!("Http client error: {}", s.clone()),
            Error::Command(ref e)          => format!("Unknown Command: {}", e.clone()),
            Error::FromUtf8Error(ref e)    => format!("From utf8 error: {}", e.clone()),
            Error::HyperError(ref e)       => format!("Hyper error: {}", e.clone()),
            Error::IoError(ref e)          => format!("IO error: {}", e.clone()),
            Error::JsonDecoderError(ref e) => format!("Failed to decode JSON: {}", e.clone()),
            Error::JsonEncoderError(ref e) => format!("Failed to encode JSON: {}", e.clone()),
            Error::PoisonError(ref e)      => format!("Poison error, {}", e.clone()),
            Error::PackageError(ref s)     => s.clone(),
            Error::ParseError(ref s)       => s.clone(),
            Error::SendErrorEvent(ref s)   => format!("Send error for Event: {}", s.clone()),
            Error::TomlDecodeError(ref e)  => format!("Toml decode error: {}", e.clone()),
            Error::TomlParserErrors(ref e) => format!("Toml parser errors: {:?}", e.clone()),
            Error::UrlParseError(ref s)    => format!("Url parse error: {}", s.clone()),
            Error::WebsocketError(ref e)   => format!("Websocket Error{:?}", e.clone()),
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
