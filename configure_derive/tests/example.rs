extern crate configure;
extern crate serde;

#[macro_use] extern crate configure_derive;
#[macro_use] extern crate serde_derive;

use std::env;
use std::path::PathBuf;
use std::net::SocketAddr;

use configure::Configure;

#[derive(Configure, Deserialize)]
#[configure(name = "example")]
#[configure(generate_docs)]
#[serde(default)]
pub struct Config {
    #[configure(docs = "This is a socket address.")]
    socket_addr: SocketAddr,
    /// This is the cert path.
    tls_cert: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            socket_addr: "127.0.0.1:7878".parse().unwrap(),
            tls_cert: None,
        }
    }
}

#[test]
fn check_that_it_works() {
    env::remove_var("CARGO_MANIFEST_DIR");
    env::set_var("EXAMPLE_TLS_CERT", "etc/certificate");

    let cfg = Config::generate().unwrap();

    assert_eq!(cfg.socket_addr, "127.0.0.1:7878".parse().unwrap());
    assert_eq!(cfg.tls_cert, Some("etc/certificate".into()));
}
