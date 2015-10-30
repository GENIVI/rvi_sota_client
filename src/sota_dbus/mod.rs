//! Implements the DBus interface to the Software Loading Manager
//!
//! Also see the documentation of the Rust [`dbus`](../../dbus/index.html) bindings.

mod sender;
mod receiver;

pub use self::sender::{send_notify, request_install, request_report};
pub use self::receiver::Receiver;
