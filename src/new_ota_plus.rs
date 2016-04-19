use rustc_serialize::json;
use std::fs::File;
use std::path::PathBuf;

use datatype::{AccessToken, Config, Error, Url, UpdateRequestId,
               UpdateReport, UpdateReportWithVin, Method, Package};
use http_client::{Auth, HttpClient2, HttpRequest2};


fn vehicle_endpoint(config: &Config, s: &str) -> Url {
    config.ota.server.join(&format!("/api/v1/vehicles/{}/{}", config.auth.vin, s)).unwrap()
}

pub fn download_package_update(config: &Config,
                               client: &HttpClient2,
                               token:  &AccessToken,
                               id:     &UpdateRequestId) -> Result<PathBuf, Error> {

    let req = HttpRequest2 {
        method: &Method::Get,
        url:    &vehicle_endpoint(config, &format!("updates/{}/download", id)),
        auth:   &Auth::Token(token),
        body:   None,
    };

    let mut path = PathBuf::new();
    path.push(&config.ota.packages_dir);
    path.push(id);
    path.set_extension(config.ota.package_manager.extension());

    let mut file = try!(File::create(path.as_path()));

    try!(client.send_request_to(&req, &mut file));

    return Ok(path)

}

pub fn send_install_report(config: &Config,
                           client: &HttpClient2,
                           token:  &AccessToken,
                           report: &UpdateReport) -> Result<(), Error> {

    let report_with_vin = UpdateReportWithVin::new(&config.auth.vin, &report);
    let json            = try!(json::encode(&report_with_vin));

    let req = HttpRequest2 {
        method: &Method::Post,
        url:    &vehicle_endpoint(config, &format!("/updates/{}", report.update_id)),
        auth:   &Auth::Token(token),
        body:   Some(&json)
    };

    let _: String = try!(client.send_request(&req));

    return Ok(())

}

pub fn get_package_updates(config: &Config,
                           client: &HttpClient2,
                           token:  &AccessToken) -> Result<Vec<UpdateRequestId>, Error> {

    let req = HttpRequest2 {
        method: &Method::Get,
        url:    &vehicle_endpoint(&config, "/updates"),
        auth:   &Auth::Token(token),
        body:   None,
    };

    let resp = try!(client.send_request(&req));

    return Ok(try!(json::decode::<Vec<UpdateRequestId>>(&resp)));

}

pub fn post_packages(config: &Config,
                     client: &HttpClient2,
                     token:  &AccessToken,
                     pkgs:   &Vec<Package>) -> Result<(), Error> {

    let json = try!(json::encode(&pkgs));

    let req = HttpRequest2 {
        method: &Method::Post,
        url:    &vehicle_endpoint(config, "/updates"),
        auth:   &Auth::Token(token),
        body:   Some(&json),
    };

    let _: String = try!(client.send_request(&req));

    return Ok(())
}

/*

#[cfg(test)]
mod tests {

    use std::io::Write;

    use super::*;
    use datatype::AccessToken;
    use datatype::{Config, OtaConfig};
    use datatype::Error;
    use datatype::Package;
    use http_client::BadHttpClient;
    use http_client::{HttpRequest, HttpClient};


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

    struct MockClient;

    impl HttpClient for MockClient {

        fn new() -> MockClient {
            MockClient
        }

        fn send_request(&self, _: &HttpRequest) -> Result<String, Error> {
            return Ok("[\"pkgid\"]".to_string())
        }

        fn send_request_to<W: Write>(&self, _: &HttpRequest, _: W) -> Result<(), Error> {
            return Ok(())
        }

    }

    #[test]
    fn test_post_packages_sends_authentication() {
        assert_eq!(
            post_packages::<MockClient>(&test_token(), &Config::default(), &vec![test_package()])
                .unwrap(), ())
    }

    #[test]
    fn test_get_package_updates() {
        assert_eq!(get_package_updates::<MockClient>(&test_token(), &Config::default()).unwrap(),
                   vec!["pkgid".to_string()])
    }

    #[test]
    #[ignore] // TODO: docker daemon requires user namespaces for this to work
    fn bad_packages_dir_download_package_update() {
        let mut config = Config::default();
        config.ota = OtaConfig { packages_dir: "/".to_string(), .. config.ota };

        assert_eq!(
            format!("{}",
                    download_package_update::<MockClient>(&test_token(), &config, &"0".to_string())
                    .unwrap_err()),
            r#"Ota error, failed to create file "/0.deb": Permission denied (os error 13)"#)
    }

    #[test]
    fn bad_client_download_package_update() {
        assert_eq!(
            format!("{}",
                    download_package_update::<BadHttpClient>
                    (&test_token(), &Config::default(), &"0".to_string())
                    .unwrap_err()),
            r#"Ota error, the request: GET http://127.0.0.1:8080/api/v1/vehicles/V1234567890123456/updates/0/download,
results in the following error: bad client."#)
    }

}

*/
