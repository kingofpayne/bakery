use hex_literal::hex;
mod common;
use common::{test_compile, test_compile_ser};
use std::collections::HashMap;

#[test]
fn test_map() {
    let rec = "x: Map<u8, i32>";
    test_compile(rec, "x: {}", &hex!("0000000000000000"));
    test_compile(rec, "x: {4: 9876}", &hex!("01000000000000000494260000"));
    let data: HashMap<u8, i32> = [(10, 123), (20, 456), (30, 789)].iter().cloned().collect();
    test_compile_ser(rec, "x: {10: 123, 20: 456, 30: 789}", None, data);
}
