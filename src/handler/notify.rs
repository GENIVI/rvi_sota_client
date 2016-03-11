//! Handles "Notify" messages.

use std::sync::Mutex;

use event::inbound::{InboundEvent, UpdateAvailable};
use message::BackendServices;
use handler::{Result, RemoteServices, HandleMessageParams};
use persistence::Transfers;


/// Type for "Notify" messages.
#[derive(RustcDecodable, Clone)]
pub struct NotifyParams {
    /// A `Vector` of packages, that are available for download.
    pub update_available: UpdateAvailable,
    /// The service URLs, that the SOTA server supports.
    pub services: BackendServices,
}

impl HandleMessageParams for NotifyParams {
    fn handle(&self,
              services: &Mutex<RemoteServices>,
              _: &Mutex<Transfers>) -> Result {
        let mut services = services.lock().unwrap();
        services.set(self.services.clone());

        Ok(Some(InboundEvent::UpdateAvailable(self.update_available.clone())))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use test_library::get_empty_backend;

    use std::sync::Mutex;

    use message::{BackendServices, PackageId, UserPackage, Notification};
    use handler::HandleMessageParams;
    use persistence::Transfers;

    use rand;
    use rand::Rng;

    fn gen_packages(i: usize) -> Vec<UserPackage> {
        let mut packages = Vec::new();

        for j in 1..i {
            let package = PackageId {
                name: rand::thread_rng()
                    .gen_ascii_chars().take(j).collect::<String>(),
                version: rand::thread_rng()
                    .gen_ascii_chars().take(j).collect::<String>(),
            };

            let notify_package = UserPackage {
                package: package,
                size: j as u64
            };

            packages.push(notify_package);
        }

        print!("Using package list: ");
        for package in &packages {
            print!("{}, ", package);
        }
        print!("\n");

        packages
    }

    #[test]
    fn it_sets_backendservices() {
        test_init!();
        for i in 1..20 {
            let services_old = Mutex::new(get_empty_backend());

            let start = rand::thread_rng()
                .gen_ascii_chars().take(i).collect::<String>();
            let cancel = rand::thread_rng()
                .gen_ascii_chars().take(i).collect::<String>();
            let ack = rand::thread_rng()
                .gen_ascii_chars().take(i).collect::<String>();
            let report = rand::thread_rng()
                .gen_ascii_chars().take(i).collect::<String>();
            let packages = rand::thread_rng()
                .gen_ascii_chars().take(i).collect::<String>();

            trace!("Testing with:");
            trace!("  start: {}", start);
            trace!("  cancel: {}", cancel);
            trace!("  ack: {}", ack);
            trace!("  report: {}", report);
            trace!("  packages: {}", packages);

            let services_new = BackendServices {
                start: start.clone(),
                ack: ack.clone(),
                report: report.clone(),
                packages: packages.clone()
            };
            let notify = NotifyParams {
                packages: gen_packages(i),
                services: services_new
            };
            let transfers = Mutex::new(Transfers::new("".to_string()));
            assert!(notify.handle(&services_old, &transfers).is_ok());
            let services = services_old.lock().unwrap().svcs.iter().next().unwrap().clone();
            assert_eq!(services.start, start);
            assert_eq!(services.ack, ack);
            assert_eq!(services.report, report);
            assert_eq!(services.packages, packages);
        }
    }

    #[test]
    fn it_promotes_services() {
        test_init!();
        for i in 1..20 {
            let services_old = Mutex::new(get_empty_backend());

            let start = rand::thread_rng()
                .gen_ascii_chars().take(i).collect::<String>();
            let cancel = rand::thread_rng()
                .gen_ascii_chars().take(i).collect::<String>();
            let ack = rand::thread_rng()
                .gen_ascii_chars().take(i).collect::<String>();
            let report = rand::thread_rng()
                .gen_ascii_chars().take(i).collect::<String>();
            let packages = rand::thread_rng()
                .gen_ascii_chars().take(i).collect::<String>();

            trace!("Testing with:");
            trace!("  start: {}", start);
            trace!("  cancel: {}", cancel);
            trace!("  ack: {}", ack);
            trace!("  report: {}", report);
            trace!("  packages: {}", packages);

            let services_new = BackendServices {
                start: start.clone(),
                ack: ack.clone(),
                report: report.clone(),
                packages: packages.clone()
            };
            let notify = NotifyParams {
                packages: gen_packages(i),
                services: services_new
            };
            let transfers = Mutex::new(Transfers::new("".to_string()));
            match notify.handle(&services_old, &transfers).unwrap().unwrap() {
                Notification::Notify(m) => {
                    assert_eq!(m.services.start, start);
                    assert_eq!(m.services.ack, ack);
                    assert_eq!(m.services.report, report);
                    assert_eq!(m.services.packages, packages);
                },
                _ => panic!("Got wrong notification!")
            }
        }
    }
}
