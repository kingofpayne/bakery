use hex_literal::hex;
mod common;
use common::test_compile_ser;

#[test]
fn test_bool() {
    let rec = "bool";
    test_compile_ser(rec, "false", Some(&hex!("00")), false);
    test_compile_ser(rec, "true", Some(&hex!("01")), true);
}
