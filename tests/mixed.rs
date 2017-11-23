extern crate configure;
extern crate test_setup;

use std::env;
use std::path::PathBuf;

use configure::Configure;
use test_setup::Configuration;

#[test]
fn mixed() {
    let dir: PathBuf = env::var_os("CARGO_MANIFEST_DIR").unwrap().into();
    env::set_var("CARGO_MANIFEST_DIR", dir.join("test-setup").join("alt-toml"));
    env::set_var("TEST_FIRST_FIELD", "12");

    assert_eq!(Configuration::generate().unwrap(), Configuration {
        first_field: 12,
        second_field: String::from("Labyrinth"),
        ..Configuration::default()
    });
}
