//! Communication with the Sota HTTP server

use rustc_serialize::json;
use std::fs::File;
use std::path::PathBuf;
use std::io::Write;

use event::inbound::UpdateAvailable;
use event::outbound::{UpdateReport, UpdateResult, InstalledPackage};

use configuration::ServerConfiguration;

use super::datatype::{UpdateRequestId, Url, Error};

use super::{Auth, HttpClient, HttpRequest, HttpResponse};

fn vehicle_updates_endpoint(config: &ServerConfiguration, path: &str) -> Url {
    config.url.join(& if path.is_empty() {
        format!("/api/v1/vehicle_updates/{}", &config.vin)
    } else {
        format!("/api/v1/vehicle_updates/{}/{}", &config.vin, path)
    }).unwrap()
}

pub fn download_package_update(config: &ServerConfiguration,
                               client: &mut HttpClient,
                               id:     &UpdateRequestId) -> Result<PathBuf, Error> {

    let req = HttpRequest::get(
        vehicle_updates_endpoint(config, &format!("{}/download", id)),
        (None as Option<Auth>),
    );

    let mut path = PathBuf::new();
    path.push(&config.packages_dir);
    path.push(id);
    path.set_extension(&config.packages_extension);

    let mut file = try!(File::create(path.as_path()));
    let resp = try!(client.send_request(&req));
    let _ = file.write(resp.body.as_ref());
    Ok(path)

}

pub fn send_install_report(config: &ServerConfiguration,
                           client: &mut HttpClient,
                           report: &UpdateReport) -> Result<(), Error> {

    let report_with_vin = UpdateResult { vin: config.vin.clone(), update_report: report.clone() };
    let json            = try!(json::encode(&report_with_vin));

    let req = HttpRequest::post(
        vehicle_updates_endpoint(config, &format!("{}", report.update_id)),
        (None as Option<Auth>),
        Some(json)
    );

    let _: HttpResponse = try!(client.send_request(&req));

    Ok(())

}

#[derive(Clone, PartialEq, Eq, Debug, RustcEncodable, RustcDecodable)]
#[allow(non_snake_case)]
struct PendingUpdateRequest {
    pub requestId: UpdateRequestId,
    pub installPos: i32,
    pub packageId: Package,
    pub createdAt: String
}

use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug, PartialEq, Eq, RustcEncodable, RustcDecodable, Clone)]
struct Package {
    pub name: String,
    pub version: String
}

impl Display for Package {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{} {}", self.name, self.version)
    }
}

pub fn get_package_updates(config: &ServerConfiguration,
                           client: &mut HttpClient) -> Result<Vec<UpdateAvailable>, Error> {

    let req = HttpRequest::get(
        vehicle_updates_endpoint(&config, ""),
        (None as Option<Auth>)
    );

    let resp = try!(client.send_request(&req));
    let body = try!(String::from_utf8(resp.body));

    let req = try!(json::decode::<Vec<PendingUpdateRequest>>(&body));

    let events: Vec<UpdateAvailable> = req.iter().map(move |r| {
        let r2 = r.clone();
        UpdateAvailable {
            update_id: r2.requestId,
            signature: "signature".to_string(),
            description: format!("{}", r2.packageId),
            request_confirmation: false,
            size: 32
        }
    }).collect();

    Ok(events)
}

// XXX: Remove in favour of update_installed_packages()?
pub fn update_packages(config: &ServerConfiguration,
                       client: &mut HttpClient,
                       pkgs:   &Vec<InstalledPackage>) -> Result<(), Error> {

    let json = try!(json::encode(&pkgs));

    debug!("update_packages, json: {}", json);

    let req = HttpRequest::put(
        vehicle_updates_endpoint(config, "installed"),
        (None as Option<Auth>),
        Some(json),
    );

    let _: HttpResponse = try!(client.send_request(&req));

    Ok(())
}
