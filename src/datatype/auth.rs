use std::borrow::Cow;


/// The available authentication types for communicating with the Auth server.
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


/// For storage of the returned access token data following a successful
/// authentication.
#[derive(RustcDecodable, Debug, PartialEq, Clone, Default)]
pub struct AccessToken {
    pub access_token: String,
    pub token_type:   String,
    pub expires_in:   i32,
    pub scope:        Vec<String>
}

impl<'a> Into<Cow<'a, AccessToken>> for AccessToken {
    fn into(self) -> Cow<'a, AccessToken> {
        Cow::Owned(self)
    }
}


/// Encapsulates a `String` type for use in `Auth::Credentials`
#[derive(Clone, PartialEq, Eq, Debug, RustcEncodable, RustcDecodable)]
pub struct ClientId(pub String);

/// Encapsulates a `String` type for use in `Auth::Credentials`
#[derive(Clone, PartialEq, Eq, Debug, RustcEncodable, RustcDecodable)]
pub struct ClientSecret(pub String);

/// Encapsulates the client id and secret used during authentication.
#[derive(Clone, PartialEq, Eq, Debug, RustcEncodable, RustcDecodable)]
pub struct ClientCredentials {
    pub client_id:     ClientId,
    pub client_secret: ClientSecret,
}
