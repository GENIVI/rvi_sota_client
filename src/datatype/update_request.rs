pub type UpdateRequestId = String;

#[derive(RustcDecodable, RustcEncodable, PartialEq, Eq, Debug, Clone)]
pub enum UpdateState {
    Accepted,
    Downloading,
    Installing,
    Installed
}
