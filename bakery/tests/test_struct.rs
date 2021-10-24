use hex_literal::hex;
mod common;
use common::{test_compile, test_compile_ser};
use serde::{Deserialize, Serialize};

#[test]
fn test_basic_struct() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Vector {
        x: i32,
        y: i32,
    }

    test_compile_ser(
        "struct Vector {\n\
          x: i32,\n\
          y: i32\n\
        },\n\
        v: Vector",
        "v: {\n\
          x: 42,\n\
          y: 84\n\
        }",
        Some(&hex!("2a00000054000000")),
        Vector { x: 42, y: 84 },
    );

    // Test empty structures
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Empty {}
    test_compile_ser(
        "struct Empty { }, x: Empty",
        "x: {}",
        Some(&hex!("")),
        Empty {},
    );

    // Test inline structure
    test_compile_ser(
        "v: struct {x: i32, y: i32}",
        "v: {x: 55, y: 12}",
        Some(&hex!("370000000c000000")),
        Vector { x: 55, y: 12 },
    );
}

#[test]
fn test_generic_struct() {
    test_compile(
        "struct Vector<T> {\n\
            x: T,\n\
            y: T\n\
        },\n\
        v: Vector<i32>,
        w: Vector<bool>",
        "v: { x: 42, y: 84 },\n\
            w: { x: false, y: true }",
        &hex!("2a000000540000000001"),
    );

    test_compile(
        "struct Vector<T1, T2> {\n\
            x: T1,\n\
            y: T2\n\
        },\n\
        v: Vector<i32, bool>",
        "v: { x: 42, y: true }",
        &hex!("2a00000001"),
    );

    test_compile(
        "struct Vector<T> {\n\
            x: T,\n\
            y: T\n\
        },\n\
        v: Vector<Vector<u32>>",
        "v: { x: { x: 1, y: 2}, y: { x: 3, y: 4 } }",
        &hex!("01000000020000000300000004000000"),
    );
}
