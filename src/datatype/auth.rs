use std::borrow::Cow;

use datatype::{AccessToken, ClientId, ClientSecret};


#[derive(Clone, Debug)]
pub enum Auth {
    None,
    Credentials(ClientId, ClientSecret),
    Token(AccessToken),
}

impl<'a> Into<Cow<'a, Auth>> for Auth {
    fn into(self) -> Cow<'a, Auth> {
        Cow::Owned(self)
    }
}
