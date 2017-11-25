extern crate configure;
extern crate test_setup;

use std::env;

use configure::Configure;
use test_setup::Configuration;

#[test]
fn env_vars_set() {
    env::remove_var("CARGO_MANIFEST_DIR");

    assert_eq!(Configuration::generate().unwrap(), Configuration::default());
}
