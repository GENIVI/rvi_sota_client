pub mod inbound;
pub mod outbound;

pub type UpdateId = String;

pub enum Event {
    Inbound(inbound::InboundEvent),
    OutBound(outbound::OutBoundEvent)
}
