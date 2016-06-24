use rustc_serialize::json;

use super::datatype::{AccessToken, ClientId, ClientSecret, Error};
use super::{Auth, HttpClient, HttpRequest};

use configuration::AuthConfiguration;

pub fn authenticate(config: &AuthConfiguration, client: &mut HttpClient) -> Result<AccessToken, Error> {

    debug!("authenticate()");

    let req = HttpRequest::post::<_, _, String>(
        config.url.clone(),
        Some(Auth::Credentials(
            ClientId     { get: config.client_id.clone() },
            ClientSecret { get: config.client_secret.clone() })),
        None,
    );

    let resp = try!(client.send_request(&req));

    let body = try!(String::from_utf8(resp.body));

    debug!("authenticate, body: `{}`", body);

    Ok(try!(json::decode(&body)))

}
