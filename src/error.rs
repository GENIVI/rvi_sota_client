use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(PartialEq, Eq, Debug)]
pub enum Error {
    AuthError(String),
    ParseError(String),
    PackageError(String),
    ClientError(String)
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let inner: String = match self {
            &Error::AuthError(ref s) => s.clone(),
            &Error::ParseError(ref s) => s.clone(),
            &Error::PackageError(ref s) => s.clone(),
            &Error::ClientError(ref s) => s.clone()
        };
        write!(f, "Application Error: {}", inner)
    }
}
