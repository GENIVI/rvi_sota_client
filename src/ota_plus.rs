use rustc_serialize::json;
use std::fs::File;
use std::path::PathBuf;

use datatype::{AccessToken, Config, Error, Url, UpdateRequestId,
               UpdateReport, UpdateReportWithVin, Package};
use http_client::{Auth, HttpClient, HttpRequest};


fn vehicle_endpoint(config: &Config, s: &str) -> Url {
    config.ota.server.join(&format!("/api/v1/vehicles/{}/{}", config.auth.vin, s)).unwrap()
}

pub fn download_package_update(config: &Config,
                               client: &mut HttpClient,
                               token:  &AccessToken,
                               id:     &UpdateRequestId) -> Result<PathBuf, Error> {

    let req = HttpRequest::get(
        vehicle_endpoint(config, &format!("updates/{}/download", id)),
        Some(Auth::Token(token)),
    );

    let mut path = PathBuf::new();
    path.push(&config.ota.packages_dir);
    path.push(id);
    path.set_extension(config.ota.package_manager.extension());

    let mut file = try!(File::create(path.as_path()));

    try!(client.send_request_to(&req, &mut file));

    return Ok(path)

}

pub fn send_install_report(config: &Config,
                           client: &mut HttpClient,
                           token:  &AccessToken,
                           report: &UpdateReport) -> Result<(), Error> {

    let report_with_vin = UpdateReportWithVin::new(&config.auth.vin, &report);
    let json            = try!(json::encode(&report_with_vin));

    let req = HttpRequest::post(
        vehicle_endpoint(config, &format!("/updates/{}", report.update_id)),
        Some(Auth::Token(token)),
        Some(json)
    );

    let _: String = try!(client.send_request(&req));

    return Ok(())

}

pub fn get_package_updates(config: &Config,
                           client: &mut HttpClient,
                           token:  &AccessToken) -> Result<Vec<UpdateRequestId>, Error> {

    let req = HttpRequest::get(
        vehicle_endpoint(&config, "/updates"),
        Some(Auth::Token(token)),
    );

    let resp = try!(client.send_request(&req));

    return Ok(try!(json::decode::<Vec<UpdateRequestId>>(&resp)));

}

// XXX: This function is only used for posting installed packages? If
// so, then it might as well get those from the package manager directly
// (which is part of config).
pub fn post_packages(config: &Config,
                     client: &mut HttpClient,
                     token:  &AccessToken,
                     pkgs:   &Vec<Package>) -> Result<(), Error> {

    let json = try!(json::encode(&pkgs));

    let req = HttpRequest::post(
        vehicle_endpoint(config, "/updates"),
        Some(Auth::Token(token)),
        Some(json),
    );

    let _: String = try!(client.send_request(&req));

    return Ok(())
}


#[cfg(test)]
mod tests {

    use super::*;
    use datatype::{AccessToken, Config, OtaConfig, Package};
    use http_client::TestHttpClient;


    fn test_token() -> AccessToken {
        AccessToken {
            access_token: "token".to_string(),
            token_type: "bar".to_string(),
            expires_in: 20,
            scope: vec![]
        }
    }

    fn test_package() -> Package {
        Package {
            name: "hey".to_string(),
            version: "1.2.3".to_string()
        }
    }

    #[test]
    fn test_post_packages_sends_authentication() {
        assert_eq!(post_packages(&Config::default(),
                                 &mut TestHttpClient::from(vec![""]),
                                 &test_token(),
                                 &vec![test_package()])
                   .unwrap(), ())
    }

    #[test]
    fn test_get_package_updates() {
        assert_eq!(get_package_updates(&Config::default(),
                                       &mut TestHttpClient::from(vec![r#"["pkgid"]"#]),
                                       &test_token()).unwrap(),
                   vec!["pkgid".to_string()])
    }

    #[test]
    fn bad_packages_dir_download_package_update() {
        let mut config = Config::default();
        config.ota = OtaConfig { packages_dir: "/".to_string(), .. config.ota };

        assert_eq!(format!("{}", download_package_update(&config,
                                                         &mut TestHttpClient::from(vec![""]),
                                                         &test_token(),
                                                         &"0".to_string())
                           .unwrap_err()),
                   "IO error: Permission denied (os error 13)")
    }

    #[test]
    fn bad_client_download_package_update() {
        assert_eq!(format!("{}",
                           download_package_update(&Config::default(),
                                                   &mut TestHttpClient::new(),
                                                   &test_token(),
                                                   &"0".to_string())
                           .unwrap_err()),
                   "Http client error: GET http://127.0.0.1:8080/api/v1/vehicles/V1234567890123456/updates/0/download")
    }

}
