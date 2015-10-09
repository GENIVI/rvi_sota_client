use super::package_id::PackageId;
use rvi::Service;

#[derive(RustcEncodable, Clone)]
pub struct LocalServices {
    pub start: String,
    pub chunk: String,
    pub abort: String,
    pub finish: String,
    pub getpackages: String,
}

impl LocalServices {
    pub fn new(s: &Vec<Service>) -> LocalServices {
        let mut services = LocalServices {
            start: "".to_string(),
            chunk: "".to_string(),
            abort: "".to_string(),
            finish: "".to_string(),
            getpackages: "".to_string()
        };

        for service in s {
            let serv = &mut services;
            match service.name.as_ref() {
                "/sota/start" => serv.start = service.addr.clone(),
                "/sota/chunk" => serv.chunk = service.addr.clone(),
                "/sota/abort" => serv.abort = service.addr.clone(),
                "/sota/finish" => serv.finish = service.addr.clone(),
                "/sota/getpackages" => serv.getpackages = service.addr.clone(),
                _ => {}
            }
        }

        services
    }

    pub fn get_vin(&self) -> String {
        // TODO: rather match by regex, than on a specific url part
        self.start.split("/").nth(2).unwrap().to_string()
    }
}

#[derive(RustcEncodable)]
pub struct InitiateParams {
    pub packages: Vec<PackageId>,
    pub services: LocalServices,
    pub vin: String
}

impl InitiateParams {
    pub fn new(p: PackageId,
               s: LocalServices) -> InitiateParams {
        let vin = s.get_vin();

        InitiateParams {
            packages: vec!(p),
            services: s,
            vin: vin
        }
    }
}
