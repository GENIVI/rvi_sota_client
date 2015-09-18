use super::Service;

pub trait RVIHandler {
    fn register(&mut self, services: Vec<Service>);
}
