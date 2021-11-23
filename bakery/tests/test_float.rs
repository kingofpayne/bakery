use bakery::write_from_string_with_recipe;
use hex_literal::hex;
mod common;
use bakery::{load_from_string, Recipe};
use common::{test_compile_ser, vec_and};
use core::fmt::Debug;
use serde::{Deserialize, Serialize};

trait NaN {
    fn float_is_nan(&self) -> bool;
}

impl NaN for f32 {
    fn float_is_nan(&self) -> bool {
        self.is_nan()
    }
}

impl NaN for f64 {
    fn float_is_nan(&self) -> bool {
        self.is_nan()
    }
}

/// Similar to `test_compile_sr` for NaN float numbers, tests only a subset of the resulting bytes.
/// Length of `mask` must be equal to length of `expect`.
///
/// To retrict the generic parameter T to be f32 or f64, T must implement the trait [`NaN`].
///
/// # Arguments
///
/// * `rec` - Recipe string
/// * `expect` - Expected binary result. Untested bits must be set to 0.
/// * `mask` - Test mask applied to `expect`. Each set bit enables the corresponding bit
///   verification.
/// * `mantissa_expect` - Expected value for mantissa testing
/// * `mantissa_mask` - Mantissa testing mask
fn test_compile_nan<T>(
    rec: &str,
    expect: &[u8],
    mask: &[u8],
    mantissa_expect: &[u8],
    mantissa_mask: &[u8],
) where
    T: NaN + Recipe + Debug + Serialize + for<'a> Deserialize<'a>,
{
    let dat = "NaN";
    let mut out = Vec::<u8>::new();
    write_from_string_with_recipe(&mut out, rec, dat).unwrap();
    let mut out_masked = out.clone();
    assert_eq!(expect.len(), mask.len()); // Required as zip won't check that
    for (a, b) in out_masked.iter_mut().zip(mask) {
        *a = *a & b;
    }
    assert_eq!(out_masked, expect);
    assert!(vec_and(&out, &mantissa_mask.to_vec()) != mantissa_expect);
    assert!(bincode::deserialize::<T>(&out).unwrap().float_is_nan());
    assert!(load_from_string::<T>(dat).unwrap().float_is_nan());
}

#[test]
fn test_f32() {
    let rec = "f32";
    test_compile_ser(rec, "0", Some(&hex!("00000000")), 0f32);

    // NaN values
    // Multiple values are possible for NaN.
    // The exponent must have all bits set to 1.
    // The mantissa must not be null.
    test_compile_nan::<f32>(
        rec,
        &hex!("0000807f"),
        &hex!("0000807f"),
        &hex!("00000000"),
        &hex!("ffff7f00"),
    );

    // Infinity
    test_compile_ser(rec, "inf", Some(&hex!("0000807f")), f32::INFINITY);
    test_compile_ser(rec, "-inf", Some(&hex!("000080ff")), f32::NEG_INFINITY);

    // Test different formatting
    let pi_enc: Option<&[u8]> = Some(&hex!("db0f4940"));
    let val = 3.141592653589793f32;
    test_compile_ser(rec, "3.141592653589793", pi_enc, val);
    test_compile_ser(rec, "0.3141592653589793E1", pi_enc, val);
    test_compile_ser(rec, "0.3141592653589793e+1", pi_enc, val);
    test_compile_ser(rec, "314.1592653589793e-2", pi_enc, val);
    test_compile_ser(rec, "314.1592653589793E-2", pi_enc, val);

    // Random vectors
    test_compile_ser(
        rec,
        "-1.749639290196046e-36",
        Some(&hex!("c5d71484")),
        -1.749639290196046e-36f32,
    );
    test_compile_ser(
        rec,
        "-9.875782720277804e+31",
        Some(&hex!("f8cf9bf4")),
        -9.875782720277804e+31f32,
    );
    test_compile_ser(
        rec,
        "-3.9331892019869604e-29",
        Some(&hex!("b76f4790")),
        -3.9331892019869604e-29f32,
    );
    test_compile_ser(rec, "16801934.0", Some(&hex!("4730804b")), 16801934.0f32);
    test_compile_ser(
        rec,
        "-3.668126322435225e-14",
        Some(&hex!("9e3225a9")),
        -3.668126322435225e-14f32,
    );
    test_compile_ser(
        rec,
        "-6.528522363489747e+18",
        Some(&hex!("f133b5de")),
        -6.528522363489747e+18f32,
    );
    test_compile_ser(
        rec,
        "-2.2542724306181628e+21",
        Some(&hex!("a168f4e2")),
        -2.2542724306181628e+21f32,
    );
    test_compile_ser(
        rec,
        "1.2289376460916657e-10",
        Some(&hex!("851f072f")),
        1.2289376460916657e-10f32,
    );
    test_compile_ser(
        rec,
        "-4.476474529599428e-13",
        Some(&hex!("cc00fcaa")),
        -4.476474529599428e-13f32,
    );
    test_compile_ser(
        rec,
        "1.8521721970630853e+20",
        Some(&hex!("7ca62061")),
        1.8521721970630853e+20f32,
    );
}

#[test]
fn test_f64() {
    let rec = "f64";
    test_compile_ser(rec, "0", Some(&hex!("0000000000000000")), 0f64);

    // NaN values
    // Multiple values are possible for NaN.
    // The exponent must have all bits set to 1.
    // The mantissa must not be null.
    test_compile_nan::<f64>(
        rec,
        &hex!("000000000000f07f"),
        &hex!("000000000000f07f"),
        &hex!("0000000000000000").to_vec(),
        &hex!("ffffffffffff0f00").to_vec(),
    );

    // Infinity
    test_compile_ser(rec, "inf", Some(&hex!("000000000000f07f")), f64::INFINITY);
    test_compile_ser(
        rec,
        "-inf",
        Some(&hex!("000000000000f0ff")),
        f64::NEG_INFINITY,
    );

    // Test different formatting
    let pi_enc: Option<&[u8]> = Some(&hex!("182d4454fb210940"));
    let val = 3.141592653589793f64;
    test_compile_ser(rec, "3.141592653589793", pi_enc, val);
    test_compile_ser(rec, "0.3141592653589793E1", pi_enc, val);
    test_compile_ser(rec, "0.3141592653589793e+1", pi_enc, val);
    test_compile_ser(rec, "314.1592653589793e-2", pi_enc, val);
    test_compile_ser(rec, "314.1592653589793E-2", pi_enc, val);

    // Random vectors
    test_compile_ser(
        rec,
        "-5.564949292668117e+282",
        Some(&hex!("717a48e52e29a3fa")),
        -5.564949292668117e+282f64,
    );
    test_compile_ser(
        rec,
        "-4.6878887837531625e+171",
        Some(&hex!("379a953faa6893e3")),
        -4.6878887837531625e+171f64,
    );
    test_compile_ser(
        rec,
        "2.6791807131619444e+151",
        Some(&hex!("2ec5a27b945e605f")),
        2.6791807131619444e+151f64,
    );
    test_compile_ser(
        rec,
        "1.0246658034412323e-215",
        Some(&hex!("1085f3232d424c13")),
        1.0246658034412323e-215f64,
    );
    test_compile_ser(
        rec,
        "-9.802724658402366e+185",
        Some(&hex!("29c88d786ed68ce6")),
        -9.802724658402366e+185f64,
    );
    test_compile_ser(
        rec,
        "3.1462665965606214e+162",
        Some(&hex!("fcb62aa63bf9ab61")),
        3.1462665965606214e+162f64,
    );
    test_compile_ser(
        rec,
        "-1.0352698653335561e-104",
        Some(&hex!("7c088a3b70be57aa")),
        -1.0352698653335561e-104f64,
    );
    test_compile_ser(
        rec,
        "2.4132492880617945e-245",
        Some(&hex!("da1f334a7017250d")),
        2.4132492880617945e-245f64,
    );
    test_compile_ser(
        rec,
        "7.673860763633534e-221",
        Some(&hex!("3f603dc82ebd3b12")),
        7.673860763633534e-221f64,
    );
    test_compile_ser(
        rec,
        "4.1853666993899847e-255",
        Some(&hex!("0b635e3ff56b1f0b")),
        4.1853666993899847e-255f64,
    );
}
