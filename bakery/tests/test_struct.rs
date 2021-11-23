use hex_literal::hex;
mod common;
use bakery_derive::Recipe;
use common::{test_compile_ser};
use serde::{Deserialize, Serialize};

#[test]
fn test_basic_struct() {
    #[derive(Recipe, Debug, PartialEq, Serialize, Deserialize)]
    struct Vector {
        x: i32,
        y: i32,
    }

    test_compile_ser(
        "struct { x: i32, y: i32 }",
        "x: 42, y: 84",
        Some(&hex!("2a00000054000000")),
        Vector { x: 42, y: 84 },
    );

    // Test empty structures
    #[derive(Recipe, Debug, PartialEq, Serialize, Deserialize)]
    struct Empty {}
    test_compile_ser("struct { }", "", Some(&hex!("")), Empty {});
}

#[test]
fn test_generic_struct_1() {
    #[derive(Recipe, Debug, PartialEq, Serialize, Deserialize)]
    struct Vector<T> {
        x: T,
        y: T
    }

    #[derive(Recipe, Debug, PartialEq, Serialize, Deserialize)]
    struct Struct {
        v: Vector<i32>,
        w: Vector<bool>
    }

    test_compile_ser(
        "struct {
            struct Vector<T> {
                x: T,
                y: T
            },
            v: Vector<i32>,
            w: Vector<bool>
        }",
        "v: { x: 42, y: 84 },
            w: { x: false, y: true }",
        Some(&hex!("2a000000540000000001")),
        Struct {
            v: Vector {
                x: 42,
                y: 84
            },
            w: Vector {
                x: false,
                y: true
            }
        }
    );
}

#[test]
fn test_generic_struct_2() {
    #[derive(Recipe, Debug, PartialEq, Serialize, Deserialize)]
    struct Vector<T1, T2> {
        x: T1,
        y: T2
    }

    #[derive(Recipe, Debug, PartialEq, Serialize, Deserialize)]
    struct Struct {
        v: Vector<i32, bool>,
    }

    test_compile_ser(
        "struct {
            struct Vector<T1, T2> {
                x: T1,
                y: T2
            },
            v: Vector<i32, bool>
        }",
        "v: { x: 42, y: true }",
        Some(&hex!("2a00000001")),
        Struct {
            v: Vector {
                x: 42,
                y: true
            }
        }
    );
}

#[test]
fn test_generic_struct_nested() {
    #[derive(Recipe, Debug, PartialEq, Serialize, Deserialize)]
    struct Vector<T> {
        x: T,
        y: T
    }

    #[derive(Recipe, Debug, PartialEq, Serialize, Deserialize)]
    struct Struct {
        v: Vector<Vector<u32>>
    }

    test_compile_ser(
        "struct {
            struct Vector<T> {
                x: T,
                y: T
            },
            v: Vector<Vector<u32>>
        }",
        "v: { x: { x: 1, y: 2}, y: { x: 3, y: 4 } }",
        Some(&hex!("01000000020000000300000004000000")),
        Struct {
            v: Vector {
                x: Vector {
                    x: 1,
                    y: 2
                },
                y: Vector {
                    x: 3,
                    y: 4
                }
            }
        }
    );
}
