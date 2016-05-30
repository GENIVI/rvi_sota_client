#[derive(Clone, PartialEq, Eq, Debug, RustcEncodable, RustcDecodable)]
pub struct ClientId(pub String);

#[derive(Clone, PartialEq, Eq, Debug, RustcEncodable, RustcDecodable)]
pub struct ClientSecret(pub String);

#[derive(Clone, PartialEq, Eq, Debug, RustcEncodable, RustcDecodable)]
pub struct ClientCredentials {
    pub id:     ClientId,
    pub secret: ClientSecret,
}
