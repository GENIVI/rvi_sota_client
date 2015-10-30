//! Trait definition of `RVIHandler`

use super::Service;

/// Provides a interface to register services.
pub trait RVIHandler {
    /// Called when registering services.
    ///
    /// # Arguments
    /// * `services`: `Vector` of registered `Service`s
    fn register(&mut self, services: Vec<Service>);
}
