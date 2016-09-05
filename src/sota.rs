use rustc_serialize::json;
use std::fs::File;
use std::io;
use std::path::PathBuf;

use datatype::{Config, DeviceReport, DownloadComplete, Error, Package,
               PendingUpdateRequest, UpdateRequestId, UpdateReport, Url};
use http::Client;


/// Encapsulate the client configuration and HTTP client used for
/// software-over-the-air updates.
pub struct Sota<'c, 'h> {
    config: &'c Config,
    client: &'h Client,
}

impl<'c, 'h> Sota<'c, 'h> {
    /// Creates a new instance for Sota communication.
    pub fn new(config: &'c Config, client: &'h Client) -> Sota<'c, 'h> {
        Sota { config: config, client: client }
    }

    /// Takes a path and returns a new endpoint of the format
    /// `<Core server>/api/v1/device_updates/<uuid>/<path>`.
    pub fn endpoint(&self, path: &str) -> Url {
        let endpoint = if path.is_empty() {
            format!("/api/v1/device_updates/{}", self.config.device.uuid)
        } else {
            format!("/api/v1/device_updates/{}/{}", self.config.device.uuid, path)
        };
        self.config.core.server.join(&endpoint).expect("couldn't build endpoint url")
    }

    /// Query the Core server to identify any new package updates available.
    pub fn get_pending_updates(&mut self) -> Result<Vec<PendingUpdateRequest>, Error> {
        let resp_rx = self.client.get(self.endpoint(""), None);
        let resp    = resp_rx.recv().expect("no get_package_updates response received");
        let data    = try!(resp);
        let text    = try!(String::from_utf8(data));
        Ok(try!(json::decode::<Vec<PendingUpdateRequest>>(&text)))
    }

    /// Download a specific update from the Core server.
    pub fn download_update(&mut self, id: UpdateRequestId) -> Result<DownloadComplete, Error> {
        let resp_rx = self.client.get(self.endpoint(&format!("{}/download", id)), None);
        let resp    = resp_rx.recv().expect("no download_package_update response received");
        let data    = try!(resp);

        let mut path = PathBuf::new();
        path.push(&self.config.device.packages_dir);
        path.push(id.clone()); // TODO: Use Content-Disposition filename from request?
        let mut file = try!(File::create(path.as_path()));

        let _    = io::copy(&mut &*data, &mut file);
        let path = try!(path.to_str().ok_or(Error::Parse(format!("Path is not valid UTF-8: {:?}", path))));

        Ok(DownloadComplete {
            update_id:    id,
            update_image: path.to_string(),
            signature:    "".to_string()
        })
    }

    /// Install an update using the package manager.
    pub fn install_update(&mut self, download: DownloadComplete) -> Result<UpdateReport, UpdateReport> {
        let ref pacman = self.config.device.package_manager;
        pacman.install_package(&download.update_image).and_then(|(code, output)| {
            Ok(UpdateReport::single(download.update_id.clone(), code, output))
        }).or_else(|(code, output)| {
            Err(UpdateReport::single(download.update_id.clone(), code, output))
        })
    }

    /// Get a list of the currently installed packages from the package manager.
    pub fn get_installed_packages(&mut self) -> Result<Vec<Package>, Error> {
        Ok(try!(self.config.device.package_manager.installed_packages()))
    }

    /// Send a list of the currently installed packages to the Core server.
    pub fn send_installed_packages(&mut self, packages: &Vec<Package>) -> Result<(), Error> {
        let body    = try!(json::encode(packages));
        let resp_rx = self.client.put(self.endpoint("installed"), Some(body.into_bytes()));
        let _       = resp_rx.recv().expect("no update_installed_packages response received")
                             .map_err(|err| error!("update_installed_packages failed: {}", err));
        Ok(())
    }

    /// Send the outcome of a package update to the Core server.
    pub fn send_update_report(&mut self, update_report: &UpdateReport) -> Result<(), Error> {
        let report  = DeviceReport::new(&self.config.device.uuid, update_report);
        let body    = try!(json::encode(&report));
        let url     = self.endpoint(report.device);
        let resp_rx = self.client.post(url, Some(body.into_bytes()));
        let resp    = resp_rx.recv().expect("no send_install_report response received");
        let _       = try!(resp);
        Ok(())
    }

    /// Send system information from the device to the Core server.
    pub fn send_system_info(&mut self, body: &str) -> Result<(), Error> {
        let resp_rx = self.client.put(self.endpoint("system_info"), Some(body.as_bytes().to_vec()));
        let resp    = resp_rx.recv().expect("no send_system_info response received");
        let _       = try!(resp);
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use rustc_serialize::json;

    use super::*;
    use datatype::{Config, Package, PendingUpdateRequest};
    use http::TestClient;


    #[test]
    fn test_get_pending_updates() {
        let pending_update = PendingUpdateRequest {
            requestId: "someid".to_string(),
            installPos: 0,
            packageId: Package {
                name: "fake-pkg".to_string(),
                version: "0.1.1".to_string()
            },
            createdAt: "2010-01-01".to_string()
        };

        let json = format!("[{}]", json::encode(&pending_update).unwrap());
        let mut sota = Sota {
            config: &Config::default(),
            client: &mut TestClient::from(vec![json.to_string()]),
        };

        let updates: Vec<PendingUpdateRequest> = sota.get_pending_updates().unwrap();
        let ids: Vec<String> = updates.iter().map(|p| p.requestId.clone()).collect();
        assert_eq!(ids, vec!["someid".to_string()])
    }
}
