use std::borrow::Cow;


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


#[derive(Clone, PartialEq, Eq, Debug, RustcEncodable, RustcDecodable)]
pub struct ClientId(pub String);

#[derive(Clone, PartialEq, Eq, Debug, RustcEncodable, RustcDecodable)]
pub struct ClientSecret(pub String);

#[derive(Clone, PartialEq, Eq, Debug, RustcEncodable, RustcDecodable)]
pub struct ClientCredentials {
    pub id:     ClientId,
    pub secret: ClientSecret,
}
