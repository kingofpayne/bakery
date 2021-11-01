use hex_literal::hex;
mod common;
use common::test_compile_ser;
use std::collections::HashMap;

#[test]
fn test_map() {
    let rec = "Map<u8, i32>";
    test_compile_ser(rec, "{}", Some(&hex!("0000000000000000")),
        [].iter().cloned().collect::<HashMap<u8, i32>>());
    test_compile_ser(rec, "{4: 9876}", Some(&hex!("01000000000000000494260000")),
        [(4, 9876)].iter().cloned().collect::<HashMap<u8, i32>>());
    // For the following, the binary is not predictable since HashMap ordering is not.
    test_compile_ser(rec, "{10: 123, 20: 456, 30: 789}", None,
        [(10, 123), (20, 456), (30, 789)].iter().cloned().collect::<HashMap<u8, i32>>());
}
