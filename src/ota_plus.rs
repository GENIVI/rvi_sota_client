use hyper::Url;
use hyper::header::{Authorization, Bearer, ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use rustc_serialize::json;
use std::fs::File;
use std::path::PathBuf;
use std::result::Result;

use datatype::access_token::AccessToken;
use datatype::config::OtaConfig;
use datatype::error::Error;
use datatype::error::OtaReason::{CreateFile, Client};
use datatype::package::Package;
use datatype::update_request::UpdateRequestId;
use http_client::{HttpClient, HttpRequest};


fn vehicle_endpoint(config: &OtaConfig, s: &str) -> Url {
    config.server.join(&format!("/api/v1/vehicles/{}{}", config.vin, s)).unwrap()
}

pub fn download_package_update<C: HttpClient>(token:  &AccessToken,
                                              config: &OtaConfig,
                                              id:     &UpdateRequestId) -> Result<PathBuf, Error> {

    let req = HttpRequest::get(vehicle_endpoint(config, &format!("/updates/{}/download", id)))
        .with_header(Authorization(Bearer { token: token.access_token.clone() }));

    let mut path = PathBuf::new();
    path.push(&config.packages_dir);
    path.push(id);
    path.set_extension("deb");

    let file = try!(File::create(path.as_path())
                    .map_err(|e| Error::Ota(CreateFile(path.clone(), e))));

    try!(C::new().send_request_to(&req, file)
         .map_err(|e| Error::Ota(Client(req.to_string(), format!("{}", e)))));

    return Ok(path)
}

pub fn get_package_updates<C: HttpClient>(token:  &AccessToken,
                                          config: &OtaConfig) -> Result<Vec<UpdateRequestId>, Error> {

    let req = HttpRequest::get(vehicle_endpoint(&config, "/updates"))
        .with_header(Authorization(Bearer { token: token.access_token.clone() }));

    let body = try!(C::new().send_request(&req)
                    .map_err(|e| Error::ClientError(format!("Can't consult package updates: {}", e))));

    return Ok(try!(json::decode::<Vec<UpdateRequestId>>(&body)));

}

pub fn post_packages<C: HttpClient>(token:  &AccessToken,
                                    config: &OtaConfig,
                                    pkgs:   &Vec<Package>) -> Result<(), Error> {

    let json = try!(json::encode(&pkgs)
                    .map_err(|_| Error::ParseError(String::from("JSON encoding error"))));

    let req = HttpRequest::post(vehicle_endpoint(config, "/updates"))
        .with_header(Authorization(Bearer { token: token.access_token.clone() }))
        .with_header(ContentType(Mime(
            TopLevel::Application,
            SubLevel::Json,
            vec![(Attr::Charset, Value::Utf8)])))
        .with_body(&json);

    let _: String = try!(C::new().send_request(&req));

    return Ok(())
}

#[cfg(test)]
mod tests {

    use std::io::Write;

    use super::*;
    use bad_http_client::BadHttpClient;
    use datatype::access_token::AccessToken;
    use datatype::config::OtaConfig;
    use datatype::error::Error;
    use datatype::package::Package;
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
            post_packages::<MockClient>(&test_token(), &OtaConfig::default(), &vec![test_package()])
                .unwrap(), ())
    }

    #[test]
    fn test_get_package_updates() {
        assert_eq!(get_package_updates::<MockClient>(&test_token(), &OtaConfig::default()).unwrap(),
                   vec!["pkgid".to_string()])
    }

    #[test]
    fn bad_packages_dir_download_package_update() {

        let mut config = OtaConfig::default();
        config = OtaConfig { packages_dir: "/".to_string(), .. config };

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
                    (&test_token(), &OtaConfig::default(), &"0".to_string())
                    .unwrap_err()),
            r#"Ota error, the request: GET http://127.0.0.1:8080/api/v1/vehicles/V1234567890123456/updates/0/download,
results in the following error: bad client."#)
    }

}
