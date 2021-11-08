use hex_literal::hex;
mod common;
use common::{test_compile, test_compile_ser};
use serde::{Deserialize, Serialize};
use bakery_derive::Recipe;

#[test]
fn test_basic_struct() {
    #[derive(Recipe, Debug, PartialEq, Serialize, Deserialize)]
    struct Vector {
        x: i32,
        y: i32,
    }

    test_compile_ser(
        "struct { x: i32, y: i32 }",
        "{ x: 42, y: 84 }",
        Some(&hex!("2a00000054000000")),
        Vector { x: 42, y: 84 },
    );

    // Test empty structures
    #[derive(Recipe, Debug, PartialEq, Serialize, Deserialize)]
    struct Empty {}
    test_compile_ser(
        "struct { }",
        "{}",
        Some(&hex!("")),
        Empty {},
    );
}

#[test]
fn test_generic_struct() {
    test_compile(
        "struct {
            struct Vector<T> {
                x: T,
                y: T
            },
            v: Vector<i32>,
            w: Vector<bool>
        }",
        "{
            v: { x: 42, y: 84 },
            w: { x: false, y: true }
        }",
        &hex!("2a000000540000000001"),
    );

    test_compile(
        "struct {
            struct Vector<T1, T2> {
                x: T1,
                y: T2
            },
            v: Vector<i32, bool>
        }",
        "{ v: { x: 42, y: true } }",
        &hex!("2a00000001"),
    );

    test_compile(
        "struct {
            struct Vector<T> {
                x: T,
                y: T
            },
            v: Vector<Vector<u32>>
        }",
        "{ v: { x: { x: 1, y: 2}, y: { x: 3, y: 4 } } }",
        &hex!("01000000020000000300000004000000"),
    );
}
