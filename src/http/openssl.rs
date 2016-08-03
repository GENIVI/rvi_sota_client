use hyper::net::Openssl;
use openssl::ssl::{SSL_OP_NO_SSLV2, SSL_OP_NO_SSLV3};
use openssl::ssl::{SslContext, SslMethod};
use std::path::Path;
use std::sync::{Arc, Mutex};


lazy_static! {
    static ref OPENSSL: Arc<Mutex<Option<Openssl>>> = Arc::new(Mutex::new(None));
}

// default cipher list taken from the Servo project:
// https://github.com/servo/servo/blob/master/components/net/connector.rs#L18
const DEFAULT_CIPHERS: &'static str = concat!(
    "ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:",
    "ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:",
    "DHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES128-SHA256:",
    "ECDHE-RSA-AES128-SHA256:ECDHE-ECDSA-AES256-SHA384:ECDHE-RSA-AES256-SHA384:",
    "ECDHE-ECDSA-AES128-SHA:ECDHE-RSA-AES128-SHA:ECDHE-ECDSA-AES256-SHA:",
    "ECDHE-RSA-AES256-SHA:DHE-RSA-AES128-SHA256:DHE-RSA-AES128-SHA:",
    "DHE-RSA-AES256-SHA256:DHE-RSA-AES256-SHA:ECDHE-RSA-DES-CBC3-SHA:",
    "ECDHE-ECDSA-DES-CBC3-SHA:AES128-GCM-SHA256:AES256-GCM-SHA384:",
    "AES128-SHA256:AES256-SHA256:AES128-SHA:AES256-SHA"
);

pub fn set_ca_certificates(path: &Path) {
    info!("Setting OpenSSL CA certificates path to {:?}", path);
    let mut openssl = OPENSSL.lock().unwrap();
    let mut context = SslContext::new(SslMethod::Sslv23).unwrap();
    context.set_CA_file(path).unwrap_or_else(|err| {
        panic!("couldn't set CA certificates: {}", err);
    });
    context.set_cipher_list(DEFAULT_CIPHERS).unwrap();
    context.set_options(SSL_OP_NO_SSLV2 | SSL_OP_NO_SSLV3);
    *openssl = Some(Openssl { context: context });
}

pub fn get_openssl() -> Openssl {
    if let Some(ref openssl) = *OPENSSL.lock().unwrap() {
        openssl.clone()
    } else {
        panic!("CA certificates not set")
    }
}
