#[derive(RustcEncodable)]
pub struct InitiateParams {
    pub id: u32,
    pub package: String,
}

#[derive(RustcEncodable, RustcDecodable)]
pub struct AckParams {
    pub id: u32,
    pub ack: String
}

#[derive(RustcEncodable, RustcDecodable)]
pub struct AckChunkParams {
    pub id: u32,
    pub ack: String,
    pub index: i32
}

// Not Encodable to prevent sending `Variant` messages
pub enum GenericAck {
    Ack(AckParams),
    Chunk(AckChunkParams)
}
