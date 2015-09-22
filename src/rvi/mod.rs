mod edge;
mod send;
mod message;
mod handler;

// Export public interface
pub use rvi::edge::ServiceEdge;
pub use rvi::edge::Service;
pub use rvi::handler::RVIHandler;
pub use rvi::send::send;
pub use rvi::send::send_message;
pub use rvi::message::Message;
