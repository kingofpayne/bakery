use hex_literal::hex;
mod common;
use common::test_compile_ser;

#[test]
fn test_bool() {
    let rec = "x: bool";
    test_compile_ser(rec, "x: false", Some(&hex!("00")), false);
    test_compile_ser(rec, "x: true", Some(&hex!("01")), true);
}
