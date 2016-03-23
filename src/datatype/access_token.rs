
#[derive(RustcDecodable, Debug, PartialEq)]
pub struct AccessToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i32,
    pub scope: Vec<String>
}
