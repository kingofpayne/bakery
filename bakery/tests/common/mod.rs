use bakery::{Recipe, load_from_string, write_from_string_with_recipe};
use core::fmt::Debug;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;

/// AND two vectors of bytes
pub fn vec_and(a: &Vec<u8>, b: &Vec<u8>) -> Vec<u8> {
    assert_eq!(a.len(), b.len());
    let mut result = a.to_vec();
    for (x, y) in result.iter_mut().zip(b) {
        *x &= y;
    }
    result
}

/// Compile data according to a recipe and check the binary result
///
/// # Arguments
///
/// * `rec` - Recipe string
/// * `dat` - Data string
/// * `expect` - Expected binary result
pub fn test_compile(rec: &str, dat: &str, expect: &[u8]) {
    let mut out = Vec::<u8>::new();
    write_from_string_with_recipe(&mut out, rec, dat).unwrap();
    assert_eq!(out, expect);
}

/// Similar to `test_compile` but test only a subset of the resulting bytes
/// Length of `mask` must be equal to length of `expect`.
///
/// # Arguments
///
/// * `rec` - Recipe string
/// * `dat` - Data string
/// * `expect` - Expected binary result. Untested bits must be set to 0.
/// * `mask` - Test mask applied to `expect`. Each set bit enables the corresponding bit
///   verification.
pub fn test_compile_mask(rec: &str, dat: &str, expect: &[u8], mask: &[u8]) -> Vec<u8> {
    let mut out = Vec::<u8>::new();
    write_from_string_with_recipe(&mut out, rec, dat).unwrap();
    let mut out_masked = out.clone();
    assert_eq!(expect.len(), mask.len()); // Required as zip won't check that
    for (a, b) in out_masked.iter_mut().zip(mask) {
        *a = *a & b;
    }
    assert_eq!(out_masked, expect);
    out
}

/// Compile data according to a recipe, check the binary result, check serialization and deserialization.
/// Panics if a test fails.
///
/// # Arguments
///
/// * `rec` - Recipe string
/// * `dat` - Data string
/// * `bin` - Expected binary result, or None if serialization is not deterministic (HashMap for instance)
/// * `val` - Expected deserialization result
pub fn test_compile_ser<T>(rec: &str, dat: &str, bin: Option<&[u8]>, val: T)
where
    T: Recipe + Debug + Serialize + PartialEq + for<'a> Deserialize<'a>,
{
    let mut out = Vec::<u8>::new();
    write_from_string_with_recipe(&mut out, rec, dat).unwrap();
    if let Some(bin) = bin {
        assert_eq!(out, bin);
        assert_eq!(bincode::serialize(&val).unwrap(), bin);
    }
    assert_eq!(bincode::deserialize::<T>(&out).unwrap(), val);
    assert_eq!(load_from_string::<T>(dat), val);
}

/// Tests loading data from a string using Recipe trait
///
/// # Arguments
///
/// * `expected` - Expected data
/// * `dat` - Loaded data string
pub fn test_load_from_string<T>(dat: &str, expected: T)
where
    T: Recipe + Debug + PartialEq + for<'a> Deserialize<'a>
{
    assert_eq!(
        load_from_string::<T>(dat),
        expected
    )
}
