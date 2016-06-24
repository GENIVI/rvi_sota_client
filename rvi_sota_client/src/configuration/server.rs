//! Handles the `server` section of the configuration file.

use toml;

use std::borrow::Cow;

use super::common::{get_required_key, ConfTreeParser, Result};
use remote::http::{Auth, Url};
use remote::http::datatype::{ClientId, ClientSecret};

impl<'a> Into<Cow<'a, Auth>> for AuthConfiguration {
    fn into(self) -> Cow<'a, Auth> {
        Cow::Owned(Auth::Credentials(ClientId { get: self.client_id },
                                     ClientSecret { get: self.client_secret }))
    }
}

#[derive(Clone, Debug)]
pub struct AuthConfiguration {
    pub client_id: String,
    pub client_secret: String,
    pub url: Url,
}

/// Type to encode allowed keys for the `server` section of the configuration.
#[derive(Clone)]
pub struct ServerConfiguration {
    pub url: Url,
    pub polling_interval: i64,
    pub vin: String,
    pub packages_dir: String,
    pub packages_extension: String,
    pub auth: Option<AuthConfiguration>
}

impl ConfTreeParser<Option<ServerConfiguration>> for ServerConfiguration {
    fn parse(tree: &toml::Table) -> Result<Option<ServerConfiguration>> {
        tree.get("server").map_or_else(|| Ok(None), |server_tree| {
            let url = try!(get_required_key(server_tree, "url", "server"));
            let polling_interval = try!(get_required_key(server_tree, "polling_interval", "server"));
            let vin = try!(get_required_key(server_tree, "vin", "server"));
            let packages_dir = try!(get_required_key(server_tree, "packages_dir", "server"));
            let packages_extension = try!(get_required_key(server_tree, "packages_extension", "server"));

            let auth = server_tree.as_table()
                .and_then(|st| st.get("auth"))
                .and_then(|auth_tree| {
                    get_required_key(auth_tree, "client_id", "auth").ok().and_then(|client_id| {
                    get_required_key(auth_tree, "client_secret", "auth").ok().and_then(|client_secret| {
                    get_required_key(auth_tree, "url", "auth").ok().map(|url| {
                        AuthConfiguration {
                            client_id: client_id,
                            client_secret: client_secret,
                            url: url,
                        }
                    })
                    })
                    })
                });

            info!("Getting {:?}", auth);
            Ok(Some(ServerConfiguration {
                url: url,
                polling_interval: polling_interval,
                vin: vin,
                packages_dir: packages_dir,
                packages_extension: packages_extension,
                auth: auth
            }))
        })
    }
}

