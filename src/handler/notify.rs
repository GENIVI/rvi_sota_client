use std::fmt;
use std::sync::Mutex;
use std::collections::HashMap;
use message::{BackendServices, PackageId, UserMessage, UserPackage};
use handler::HandleMessageParams;
use persistence::Transfer;

impl fmt::Display for UserPackage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, size: {}", self.package, self.size)
    }
}

#[derive(RustcDecodable, Clone)]
pub struct NotifyParams {
    pub packages: Vec<UserPackage>,
    pub services: BackendServices
}

impl HandleMessageParams for NotifyParams {
    fn handle(&self,
              services: &Mutex<BackendServices>,
              _: &Mutex<HashMap<PackageId, Transfer>>,
              _: &str, _: &str, _: &str) -> bool {
        let mut services = services.lock().unwrap();
        services.update(&self.services);

        for package in &self.packages {
            info!("New package available: {}", package);
        }

        true
    }

    fn get_message(&self) -> Option<UserMessage> {
        Some(UserMessage {
            packages: self.packages.clone(),
            services: self.services.clone()
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::collections::HashMap;
    use std::sync::Mutex;

    use message::{BackendServices, PackageId, UserPackage};
    use handler::HandleMessageParams;
    use persistence::Transfer;

    use rand;
    use rand::Rng;

    fn get_empty_backend() -> BackendServices {
        BackendServices {
            start: "".to_string(),
            cancel: "".to_string(),
            ack: "".to_string(),
            report: "".to_string()
        }
    }

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

            trace!("Testing with:");
            trace!("  start: {}", start);
            trace!("  cancel: {}", cancel);
            trace!("  ack: {}", ack);
            trace!("  report: {}", report);

            let services_new = BackendServices {
                start: start.clone(),
                cancel: cancel.clone(),
                ack: ack.clone(),
                report: report.clone()
            };
            let notify = NotifyParams {
                packages: gen_packages(i),
                services: services_new
            };
            let transfers = Mutex::new(HashMap::<PackageId, Transfer>::new());
            assert!(notify.handle(&services_old, &transfers, "", "", ""));
            let services = services_old.lock().unwrap();
            assert_eq!(services.start, start);
            assert_eq!(services.cancel, cancel);
            assert_eq!(services.ack, ack);
            assert_eq!(services.report, report);
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

            trace!("Testing with:");
            trace!("  start: {}", start);
            trace!("  cancel: {}", cancel);
            trace!("  ack: {}", ack);
            trace!("  report: {}", report);

            let services_new = BackendServices {
                start: start.clone(),
                cancel: cancel.clone(),
                ack: ack.clone(),
                report: report.clone()
            };
            let notify = NotifyParams {
                packages: gen_packages(i),
                services: services_new
            };
            let transfers = Mutex::new(HashMap::<PackageId, Transfer>::new());
            assert!(notify.handle(&services_old, &transfers, "", "", ""));
            let promoted = notify.get_message().unwrap();
            assert_eq!(promoted.services.start, start);
            assert_eq!(promoted.services.cancel, cancel);
            assert_eq!(promoted.services.ack, ack);
            assert_eq!(promoted.services.report, report);
        }
    }

    #[test]
    fn it_promotes_packages() {
        test_init!();
        for i in 1..20 {
            let packages = gen_packages(i);
            let services = get_empty_backend();
            let notify = NotifyParams {
                packages: packages.clone(),
                services: services.clone()
            };
            let promoted = notify.get_message().unwrap();

            assert_eq!(promoted.packages, packages);
        }
    }
}
