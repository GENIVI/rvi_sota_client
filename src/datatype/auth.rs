use std::borrow::Cow;


/// The available authentication types for communicating with the Auth server.
#[derive(Clone, Debug)]
pub enum Auth {
    None,
    Credentials(ClientCredentials),
    Token(AccessToken),
}

impl<'a> Into<Cow<'a, Auth>> for Auth {
    fn into(self) -> Cow<'a, Auth> {
        Cow::Owned(self)
    }
}


/// Encapsulates the client id and secret used during authentication.
#[derive(Clone, PartialEq, Eq, Debug, RustcEncodable, RustcDecodable)]
pub struct ClientCredentials {
    pub client_id:     String,
    pub client_secret: String,
}


/// Stores the returned access token data following a successful authentication.
#[derive(RustcDecodable, Debug, PartialEq, Clone, Default)]
pub struct AccessToken {
    pub access_token: String,
    pub token_type:   String,
    pub expires_in:   i32,
    pub scope:        String
}

impl<'a> Into<Cow<'a, AccessToken>> for AccessToken {
    fn into(self) -> Cow<'a, AccessToken> {
        Cow::Owned(self)
    }
}
