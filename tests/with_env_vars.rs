extern crate configure;
extern crate test_setup;

use std::env;

use configure::Configure;
use test_setup::Configuration;

#[test]
fn env_vars_set() {
    env::remove_var("CARGO_MANIFEST_DIR");
    env::set_var("TEST_FIRST_FIELD", "7");
    env::set_var("TEST_SECOND_FIELD", "BazQuux");
    env::set_var("TEST_THIRD_FIELD", "0,1");

    assert_eq!(Configuration::generate().unwrap(), Configuration {
        first_field: 7,
        second_field: String::from("BazQuux"),
        third_field: Some(vec![0, 1]),
    });
}
