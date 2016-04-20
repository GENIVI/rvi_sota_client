use std::borrow::Cow;
use hyper::client::IntoUrl;
use hyper;
use url;
use url::ParseError;

use datatype::Error;


#[derive(RustcDecodable, PartialEq, Eq, Clone, Debug)]
pub struct Url {
    get: url::Url
}

impl Url {

    pub fn parse(s: &str) -> Result<Url, Error> {
        let url = try!(url::Url::parse(s));
        Ok(Url { get: url })
    }

    pub fn join(&self, suf: &str) -> Result<Url, Error> {
        let url = try!(self.get.join(suf));
        Ok(Url { get: url })
    }

}

impl IntoUrl for Url {

    fn into_url(self) -> Result<hyper::Url, ParseError> {
        Ok(self.get)
    }

}

impl<'a> Into<Cow<'a, Url>> for Url {
    fn into(self) -> Cow<'a, Url> {
        Cow::Owned(self)
    }
}


impl ToString for Url {

    fn to_string(&self) -> String {
        self.get.to_string()
    }

}
