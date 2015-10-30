//! Parsing of the configuration file of `sota_client`.
//!
//! Also see the documentation for [`toml`](../../toml/index.html).

mod configuration;
mod common;
mod client;
mod dbus;

pub use self::configuration::Configuration;
pub use self::client::ClientConfiguration;
pub use self::dbus::DBusConfiguration;
