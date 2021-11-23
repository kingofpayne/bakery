use hex_literal::hex;
mod common;
use bakery::load_from_string;
use bakery_derive::*;
use common::test_compile_ser;
use serde::Deserialize;

#[test]
fn test_whitespaces() {
    test_compile_ser("  i8", "  42", Some(&hex!("2a")), 42i8);
    test_compile_ser(" i8", " 42", Some(&hex!("2a")), 42i8);
    test_compile_ser("i8 ", "42 ", Some(&hex!("2a")), 42i8);
    test_compile_ser("i8  ", "42  ", Some(&hex!("2a")), 42i8);
}

#[test]
fn test_derive_simple() {
    #[derive(Deserialize, PartialEq, Eq, Debug, Recipe)]
    struct S {
        x: i32,
        y: i32,
    }

    let s = load_from_string::<S>("x: 1, y: 2").unwrap();
    assert_eq!(s, S { x: 1, y: 2 });
}

#[test]
fn test_derive_nested() {
    #[derive(Deserialize, PartialEq, Eq, Debug, Recipe)]
    struct A {
        x: i32,
        y: i32,
    }

    #[derive(Deserialize, PartialEq, Eq, Debug, Recipe)]
    struct B {
        a: A,
        b: A,
    }

    let s = load_from_string::<B>("a: { x: 1, y: 2 }, b: { x: 3, y: 4 }").unwrap();
    assert_eq!(
        s,
        B {
            a: A { x: 1, y: 2 },
            b: A { x: 3, y: 4 }
        }
    );
}

#[test]
fn test_derive_generic() {
    #[derive(Deserialize, PartialEq, Eq, Debug, Recipe)]
    struct A<T> {
        x: T,
        y: T,
    }

    #[derive(Deserialize, PartialEq, Eq, Debug, Recipe)]
    struct B {
        a: A<i32>,
        b: A<bool>,
    }

    let s = load_from_string::<B>("a: { x: 1, y: 2 }, b: { x: true, y: false }").unwrap();
    assert_eq!(
        s,
        B {
            a: A { x: 1, y: 2 },
            b: A { x: true, y: false }
        }
    );
}
