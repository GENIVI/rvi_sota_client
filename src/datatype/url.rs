use hyper::client::IntoUrl;
use hyper;
use url;
use url::ParseError;

use datatype::Error;


#[derive(RustcDecodable, PartialEq, Eq, Clone, Debug)]
pub struct Url {
    url: url::Url
}

impl Url {

    pub fn parse(s: &str) -> Result<Url, Error> {
        let url = try!(url::Url::parse(s));
        Ok(Url { url: url })
    }

    pub fn join(&self, suf: &str) -> Result<Url, Error> {
        let url = try!(self.url.join(suf));
        Ok(Url { url: url })
    }

}

impl IntoUrl for Url {

    fn into_url(self) -> Result<hyper::Url, ParseError> {
        Ok(self.url)
    }

}


impl ToString for Url {

    fn to_string(&self) -> String {
        self.url.to_string()
    }

}
