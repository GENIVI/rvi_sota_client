use std::borrow::Cow;


#[derive(RustcDecodable, Debug, PartialEq, Clone, Default)]
pub struct AccessToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i32,
    pub scope: Vec<String>
}

impl<'a> Into<Cow<'a, AccessToken>> for AccessToken {
    fn into(self) -> Cow<'a, AccessToken> {
        Cow::Owned(self)
    }
}
