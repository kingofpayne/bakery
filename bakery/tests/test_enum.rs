use hex_literal::hex;
mod common;
use bakery_derive::Recipe;
use common::{test_compile, test_compile_ser};
use serde::{Deserialize, Serialize};

#[test]
fn test_basic_enum() {
    #[derive(Recipe, Debug, PartialEq, Serialize, Deserialize)]
    enum E {
        A,
        B,
        C,
    }
    let rec = "enum { A, B, C }";
    test_compile_ser(rec, "A", Some(&hex!("00000000")), E::A);
    test_compile_ser(rec, "B", Some(&hex!("01000000")), E::B);
    test_compile_ser(rec, "C", Some(&hex!("02000000")), E::C);
}

#[test]
fn test_enum_tuple() {
    #[derive(Recipe, Debug, PartialEq, Serialize, Deserialize)]
    enum E {
        A(u32),
        B(bool),
        C(i32, bool),
    }
    let rec = "enum { A(u32), B(bool), C(i32, bool) }";
    test_compile_ser(rec, "A(0)", Some(&hex!("0000000000000000")), E::A(0));
    test_compile_ser(rec, "A(1)", Some(&hex!("0000000001000000")), E::A(1));
    test_compile_ser(
        rec,
        "A(887668751)",
        Some(&hex!("000000000fc0e834")),
        E::A(887668751),
    );
    test_compile_ser(rec, "B(true)", Some(&hex!("0100000001")), E::B(true));
    test_compile_ser(rec, "B(false)", Some(&hex!("0100000000")), E::B(false));
    test_compile_ser(
        rec,
        "C(1627069767, false)",
        Some(&hex!("02000000471dfb6000")),
        E::C(1627069767, false),
    );
    test_compile_ser(
        rec,
        "C(-453981819, true)",
        Some(&hex!("0200000085c9f0e401")),
        E::C(-453981819, true),
    );
}

#[test]
fn test_enum_struct() {
    #[derive(Recipe, Debug, PartialEq, Serialize, Deserialize)]
    enum E {
        A { a: u32 },
        B { b: bool },
        C { c: i32, d: bool },
    }
    let rec = "enum { A { a: u32 }, B { b: bool }, C { c: i32, d: bool } }";
    test_compile_ser(
        rec,
        "A { a: 42 }",
        Some(&hex!("000000002a000000")),
        E::A { a: 42 },
    );
}

#[test]
fn test_enum_empty_struct() {
    #[derive(Recipe, Debug, PartialEq, Serialize, Deserialize)]
    enum E {
        A {},
        B {},
        C {},
    }
    let rec = "enum { A { }, B { }, C { } }";
    test_compile_ser(rec, "A { }", Some(&hex!("00000000")), E::A {});
    test_compile_ser(rec, "B { }", Some(&hex!("01000000")), E::B {});
    test_compile_ser(rec, "C { }", Some(&hex!("02000000")), E::C {});
}

#[test]
fn test_inline_enum() {
    #[derive(Recipe, Debug, PartialEq, Serialize, Deserialize)]
    enum E {
        A,
        B,
        C,
    }
    let rec = "enum { A, B, C }";
    test_compile(rec, "A", &hex!("00000000"));
    test_compile(rec, "B", &hex!("01000000"));
    test_compile(rec, "C", &hex!("02000000"));
}
