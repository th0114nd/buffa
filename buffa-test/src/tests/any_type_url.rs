//! Any pack/unpack with a user-generated message's TYPE_URL const.

// #13: Any::pack/unpack with a USER-GENERATED message's TYPE_URL const.
// buffa-types tests cover Any with WKT types; this exercises the
// generated TYPE_URL const on a buffa-test message.
use crate::basic::{Address, Person};
use buffa_types::google::protobuf::Any;

#[test]
fn test_any_pack_with_generated_type_url() {
    let addr = Address {
        street: "123 Main St".into(),
        city: "Springfield".into(),
        zip_code: 12345,
        ..Default::default()
    };
    let any = Any::pack(&addr, Address::TYPE_URL);
    assert_eq!(any.type_url(), "type.googleapis.com/basic.Address");
    assert_eq!(any.type_url(), Address::TYPE_URL);
}

#[test]
fn test_any_unpack_if_type_url_matches() {
    let addr = Address {
        street: "x".into(),
        ..Default::default()
    };
    let any = Any::pack(&addr, Address::TYPE_URL);
    // unpack_if with the correct TYPE_URL succeeds.
    let decoded: Option<Address> = any.unpack_if(Address::TYPE_URL).expect("decode");
    assert_eq!(decoded, Some(addr));
}

#[test]
fn test_any_unpack_if_wrong_type_url_returns_none() {
    let addr = Address {
        street: "x".into(),
        ..Default::default()
    };
    let any = Any::pack(&addr, Address::TYPE_URL);
    // Wrong TYPE_URL — should return Ok(None), not error.
    let result: Result<Option<Person>, _> = any.unpack_if(Person::TYPE_URL);
    assert_eq!(result, Ok(None));
}

#[test]
fn test_any_is_type_with_generated_const() {
    let addr = Address::default();
    let any = Any::pack(&addr, Address::TYPE_URL);
    assert!(any.is_type(Address::TYPE_URL));
    assert!(!any.is_type(Person::TYPE_URL));
}
