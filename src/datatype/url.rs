use hyper::method;
use rustc_serialize::{Decoder, Decodable};
use std::borrow::Cow;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io;
use std::net::ToSocketAddrs;
use std::str::FromStr;
use url;
use url::SocketAddrs;

use datatype::Error;


/// Encapsulate a single crate URL with additional methods and traits.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Url(pub url::Url);

impl Url {
    /// Append the string suffix to this URL.
    pub fn join(&self, suffix: &str) -> Result<Url, Error> {
        let url = try!(self.0.join(suffix));
        Ok(Url(url))
    }

    /// Return the encapsulated crate URL.
    pub fn inner(&self) -> url::Url {
        self.0.clone()
    }
}

impl<'a> Into<Cow<'a, Url>> for Url {
    fn into(self) -> Cow<'a, Url> {
        Cow::Owned(self)
    }
}

impl FromStr for Url {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = try!(url::Url::parse(s));
        Ok(Url(url))
    }
}

impl Decodable for Url {
    fn decode<D: Decoder>(d: &mut D) -> Result<Url, D::Error> {
        let s = try!(d.read_str());
        s.parse().map_err(|e: Error| d.error(&e.to_string()))
    }
}

impl ToSocketAddrs for Url {
    type Iter = SocketAddrs;

    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        self.0.to_socket_addrs()
    }
}

impl Display for Url {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let host = self.0.host_str().unwrap_or("localhost");
        if let Some(port) = self.0.port() {
            write!(f, "{}://{}:{}{}", self.0.scheme(), host, port, self.0.path())
        } else {
            write!(f, "{}://{}{}", self.0.scheme(), host, self.0.path())
        }
    }
}


/// Enumerate the supported HTTP methods.
#[derive(Clone, Debug)]
pub enum Method {
    Get,
    Post,
    Put,
}

impl Into<method::Method> for Method {
    fn into(self) -> method::Method {
        match self {
            Method::Get  => method::Method::Get,
            Method::Post => method::Method::Post,
            Method::Put  => method::Method::Put,
        }
    }
}

impl<'a> Into<Cow<'a, Method>> for Method {
    fn into(self) -> Cow<'a, Method> {
        Cow::Owned(self)
    }
}

impl Display for Method {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let method = match *self {
            Method::Get  => "GET".to_string(),
            Method::Post => "POST".to_string(),
            Method::Put  => "PUT".to_string(),
        };
        write!(f, "{}", method)
    }
}
