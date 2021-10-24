use hex_literal::hex;
mod common;
use common::test_compile_ser;

#[test]
fn test_list() {
    test_compile_ser(
        "x: List<u8>",
        "x: [1, 2, 3, 4]",
        Some(&hex!("040000000000000001020304")),
        vec![1u8, 2, 3, 4],
    );
}
