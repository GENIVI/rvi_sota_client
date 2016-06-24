use rustc_serialize::{Decoder, Decodable};
use std::borrow::Cow;
use url;

use datatype::Error;


#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Url(pub url::Url);

impl Url {
    pub fn parse(s: &str) -> Result<Url, Error> {
        let url = try!(url::Url::parse(s));
        Ok(Url(url))
    }

    pub fn join(&self, suf: &str) -> Result<Url, Error> {
        let url = try!(self.0.join(suf));
        Ok(Url(url))
    }

    pub fn inner(&self) -> url::Url {
        self.0.clone()
    }
}

impl<'a> Into<Cow<'a, Url>> for Url {
    fn into(self) -> Cow<'a, Url> {
        Cow::Owned(self)
    }
}

impl ToString for Url {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Decodable for Url {
    fn decode<D: Decoder>(d: &mut D) -> Result<Url, D::Error> {
        let s = try!(d.read_str());
        Url::parse(&s).map_err(|e| d.error(&e.to_string()))
    }
}
