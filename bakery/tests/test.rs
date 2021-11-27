use hex_literal::hex;
use std::path::Path;
mod common;
use bakery::{load_from_file, load_from_file_with_recipe, load_from_string};
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

#[test]
fn test_load_from_file() {
    #[derive(Deserialize, PartialEq, Eq, Debug, Recipe)]
    struct Vector<T> {
        x: T,
        y: T,
    }

    #[derive(Deserialize, PartialEq, Eq, Debug, Recipe)]
    struct S {
        a: Vector<u32>,
        b: Vector<bool>,
    }

    let s: S = load_from_file_with_recipe(
        Path::new("tests")
            .join("test.rec")
            .into_os_string()
            .to_str()
            .unwrap(),
        Path::new("tests")
            .join("test.dat")
            .into_os_string()
            .to_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        s,
        S {
            a: Vector { x: 1, y: 2 },
            b: Vector { x: false, y: true }
        }
    );

    // Remove cache for next test
    std::fs::remove_file(
        Path::new("tests")
            .join("test.bin")
            .into_os_string()
            .to_str()
            .unwrap(),
    )
    .unwrap();

    let s: S = load_from_file(
        Path::new("tests")
            .join("test.dat")
            .into_os_string()
            .to_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        s,
        S {
            a: Vector { x: 1, y: 2 },
            b: Vector { x: false, y: true }
        }
    );

    // Remove cache for next test
    std::fs::remove_file(
        Path::new("tests")
            .join("test.bin")
            .into_os_string()
            .to_str()
            .unwrap(),
    )
    .unwrap();
}
