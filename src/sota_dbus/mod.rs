//! Implements the DBus interface to the Software Loading Manager
//!
//! Also see the documentation of the Rust [`dbus`](../../dbus/index.html) bindings.

mod decode;
pub mod sender;
mod receiver;

pub use self::receiver::Receiver;
