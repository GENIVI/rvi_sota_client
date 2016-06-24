//! RVI bindings for Rust.
//!
//! RVI - Remote Vehicle Interaction - is the next generation of connected vehicle services. Based
//! on the discussions inside and outside the Automotive Grade Linux expert group.
//!
//! This module implements Rust bindings to simplify the interaction with it.
//!
//! It is intended to be split out into a separate crate at some point in the future.

mod edge;
mod send;
mod message;

// Export public interface
pub use super::rvi::edge::{ServiceEdge, ServiceHandler};
pub use super::rvi::send::send;
pub use super::rvi::send::send_message;
pub use super::rvi::message::Message;
