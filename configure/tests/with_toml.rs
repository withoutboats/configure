#[macro_use] extern crate configure;
extern crate test_setup;

use std::env;
use std::path::PathBuf;

use configure::Configure;
use test_setup::Configuration;

#[test]
fn from_toml_values() {
    let dir: PathBuf = env::var_os("CARGO_MANIFEST_DIR").unwrap().into();
    env::set_var("CARGO_MANIFEST_DIR", dir.join("test-setup"));
    use_default_config!();

    assert_eq!(Configuration::generate().unwrap(), Configuration {
        first_field: 9,
        second_field: String::from("Colonel Aureliano Buendia"),
        third_field: Some(vec![10, 20, 30]),
    });
}
