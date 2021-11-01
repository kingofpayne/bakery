use hex_literal::hex;
mod common;
use common::test_compile_ser;

#[test]
fn test_i8() {
    let rec = "i8";
    test_compile_ser(rec, "0", Some(&hex!("00")), 0i8);
    test_compile_ser(rec, "-1", Some(&hex!("ff")), -1i8);
    test_compile_ser(rec, "-128", Some(&hex!("80")), -128i8);
    test_compile_ser(rec, "127", Some(&hex!("7f")), 127i8);
    // Random vectors
    test_compile_ser(rec, "-4", Some(&hex!("fc")), -4i8);
    test_compile_ser(rec, "-50", Some(&hex!("ce")), -50i8);
    test_compile_ser(rec, "25", Some(&hex!("19")), 25i8);
    test_compile_ser(rec, "122", Some(&hex!("7a")), 122i8);
    test_compile_ser(rec, "116", Some(&hex!("74")), 116i8);
    test_compile_ser(rec, "111", Some(&hex!("6f")), 111i8);
    test_compile_ser(rec, "-3", Some(&hex!("fd")), -3i8);
    test_compile_ser(rec, "19", Some(&hex!("13")), 19i8);
    test_compile_ser(rec, "-98", Some(&hex!("9e")), -98i8);
    test_compile_ser(rec, "-91", Some(&hex!("a5")), -91i8);
}

#[test]
fn test_u8() {
    let rec = "u8";
    test_compile_ser(rec, "0", Some(&hex!("00")), 0u8);
    test_compile_ser(rec, "255", Some(&hex!("ff")), 255u8);
    // Random vectors
    test_compile_ser(rec, "134", Some(&hex!("86")), 134u8);
    test_compile_ser(rec, "156", Some(&hex!("9c")), 156u8);
    test_compile_ser(rec, "127", Some(&hex!("7f")), 127u8);
    test_compile_ser(rec, "151", Some(&hex!("97")), 151u8);
    test_compile_ser(rec, "59", Some(&hex!("3b")), 59u8);
    test_compile_ser(rec, "27", Some(&hex!("1b")), 27u8);
    test_compile_ser(rec, "78", Some(&hex!("4e")), 78u8);
    test_compile_ser(rec, "227", Some(&hex!("e3")), 227u8);
    test_compile_ser(rec, "123", Some(&hex!("7b")), 123u8);
    test_compile_ser(rec, "116", Some(&hex!("74")), 116u8);
}

#[test]
fn test_i16() {
    let rec = "i16";
    test_compile_ser(rec, "0", Some(&hex!("0000")), 0i16);
    test_compile_ser(rec, "-1", Some(&hex!("ffff")), -1i16);
    test_compile_ser(rec, "-32768", Some(&hex!("0080")), -32768i16);
    test_compile_ser(rec, "32767", Some(&hex!("ff7f")), 32767i16);
    // Random vectors
    test_compile_ser(rec, "15549", Some(&hex!("bd3c")), 15549i16);
    test_compile_ser(rec, "20778", Some(&hex!("2a51")), 20778i16);
    test_compile_ser(rec, "-27791", Some(&hex!("7193")), -27791i16);
    test_compile_ser(rec, "-12227", Some(&hex!("3dd0")), -12227i16);
    test_compile_ser(rec, "-13868", Some(&hex!("d4c9")), -13868i16);
    test_compile_ser(rec, "24849", Some(&hex!("1161")), 24849i16);
    test_compile_ser(rec, "-20422", Some(&hex!("3ab0")), -20422i16);
    test_compile_ser(rec, "-1105", Some(&hex!("affb")), -1105i16);
    test_compile_ser(rec, "-28220", Some(&hex!("c491")), -28220i16);
    test_compile_ser(rec, "27364", Some(&hex!("e46a")), 27364i16);
}

#[test]
fn test_u16() {
    let rec = "u16";
    test_compile_ser(rec, "0", Some(&hex!("0000")), 0u16);
    test_compile_ser(rec, "65535", Some(&hex!("ffff")), 65535u16);
    // Random vectors
    test_compile_ser(rec, "21581", Some(&hex!("4d54")), 21581u16);
    test_compile_ser(rec, "58867", Some(&hex!("f3e5")), 58867u16);
    test_compile_ser(rec, "32137", Some(&hex!("897d")), 32137u16);
    test_compile_ser(rec, "42782", Some(&hex!("1ea7")), 42782u16);
    test_compile_ser(rec, "38907", Some(&hex!("fb97")), 38907u16);
    test_compile_ser(rec, "60739", Some(&hex!("43ed")), 60739u16);
    test_compile_ser(rec, "41645", Some(&hex!("ada2")), 41645u16);
    test_compile_ser(rec, "24172", Some(&hex!("6c5e")), 24172u16);
    test_compile_ser(rec, "11735", Some(&hex!("d72d")), 11735u16);
    test_compile_ser(rec, "33016", Some(&hex!("f880")), 33016u16);
}

#[test]
fn test_i32() {
    let rec = "i32";
    test_compile_ser(rec, "0", Some(&hex!("00000000")), 0i32);
    test_compile_ser(rec, "-1", Some(&hex!("ffffffff")), -1i32);
    test_compile_ser(
        rec,
        "-2147483648",
        Some(&hex!("00000080")),
        -2147483648i32,
    );
    test_compile_ser(rec, "2147483647", Some(&hex!("ffffff7f")), 2147483647i32);
    // Random vectors
    test_compile_ser(rec, "463957049", Some(&hex!("396ca71b")), 463957049i32);
    test_compile_ser(
        rec,
        "-1534200772",
        Some(&hex!("3cf48da4")),
        -1534200772i32,
    );
    test_compile_ser(rec, "-655069093", Some(&hex!("5b70f4d8")), -655069093i32);
    test_compile_ser(
        rec,
        "-1593580764",
        Some(&hex!("24e303a1")),
        -1593580764i32,
    );
    test_compile_ser(
        rec,
        "-2011365495",
        Some(&hex!("89ff1c88")),
        -2011365495i32,
    );
    test_compile_ser(rec, "791315362", Some(&hex!("a2832a2f")), 791315362i32);
    test_compile_ser(rec, "-395676156", Some(&hex!("04766ae8")), -395676156i32);
    test_compile_ser(rec, "477225567", Some(&hex!("5fe2711c")), 477225567i32);
    test_compile_ser(rec, "1807040406", Some(&hex!("963fb56b")), 1807040406i32);
    test_compile_ser(rec, "514118409", Some(&hex!("09d3a41e")), 514118409i32);
}

#[test]
fn test_u32() {
    let rec = "u32";
    test_compile_ser(rec, "0", Some(&hex!("00000000")), 0u32);
    test_compile_ser(rec, "4294967295", Some(&hex!("ffffffff")), 4294967295u32);
    // Random vectors
    test_compile_ser(rec, "554524088", Some(&hex!("b85d0d21")), 554524088u32);
    test_compile_ser(rec, "3826198075", Some(&hex!("3b260fe4")), 3826198075u32);
    test_compile_ser(rec, "1446776941", Some(&hex!("6d103c56")), 1446776941u32);
    test_compile_ser(rec, "2578485596", Some(&hex!("5c91b099")), 2578485596u32);
    test_compile_ser(rec, "370701113", Some(&hex!("39731816")), 370701113u32);
    test_compile_ser(rec, "181880392", Some(&hex!("4846d70a")), 181880392u32);
    test_compile_ser(rec, "1339569466", Some(&hex!("3a35d84f")), 1339569466u32);
    test_compile_ser(rec, "1637158243", Some(&hex!("630d9561")), 1637158243u32);
    test_compile_ser(rec, "3293265353", Some(&hex!("c93d4bc4")), 3293265353u32);
    test_compile_ser(rec, "2169735811", Some(&hex!("838a5381")), 2169735811u32);
}

#[test]
fn test_i64() {
    let rec = "i64";
    test_compile_ser(rec, "0", Some(&hex!("0000000000000000")), 0i64);
    test_compile_ser(rec, "-1", Some(&hex!("ffffffffffffffff")), -1i64);
    test_compile_ser(
        rec,
        "-9223372036854775808",
        Some(&hex!("0000000000000080")),
        -9223372036854775808i64,
    );
    test_compile_ser(
        rec,
        "9223372036854775807",
        Some(&hex!("ffffffffffffff7f")),
        9223372036854775807i64,
    );
    // Random vectors
    test_compile_ser(
        rec,
        "-9133791049491081848",
        Some(&hex!("881571f26e413e81")),
        -9133791049491081848i64,
    );
    test_compile_ser(
        rec,
        "-6737255667976734192",
        Some(&hex!("107a425daa7580a2")),
        -6737255667976734192i64,
    );
    test_compile_ser(
        rec,
        "3413298683106364856",
        Some(&hex!("b80ddd5e11795e2f")),
        3413298683106364856i64,
    );
    test_compile_ser(
        rec,
        "-8988785798703311961",
        Some(&hex!("a79336d2f06a4183")),
        -8988785798703311961i64,
    );
    test_compile_ser(
        rec,
        "-5986556545684688518",
        Some(&hex!("7ab1ac328a7aebac")),
        -5986556545684688518i64,
    );
    test_compile_ser(
        rec,
        "7812557482777452888",
        Some(&hex!("5821454ca3c76b6c")),
        7812557482777452888i64,
    );
    test_compile_ser(
        rec,
        "3257888572925282567",
        Some(&hex!("070904636558362d")),
        3257888572925282567i64,
    );
    test_compile_ser(
        rec,
        "-6296089866154329551",
        Some(&hex!("313aeff59fcb9fa8")),
        -6296089866154329551i64,
    );
    test_compile_ser(
        rec,
        "3619987863431847675",
        Some(&hex!("fb9e1443c8c73c32")),
        3619987863431847675i64,
    );
    test_compile_ser(
        rec,
        "-819452756873168922",
        Some(&hex!("e6375c2c0fb8a0f4")),
        -819452756873168922i64,
    );
}

#[test]
fn test_u64() {
    let rec = "u64";
    test_compile_ser(rec, "0", Some(&hex!("0000000000000000")), 0u64);
    test_compile_ser(
        rec,
        "18446744073709551615",
        Some(&hex!("ffffffffffffffff")),
        18446744073709551615u64,
    );
    // Random vectors
    test_compile_ser(
        rec,
        "14496082492297279367",
        Some(&hex!("87772324396e2cc9")),
        14496082492297279367u64,
    );
    test_compile_ser(
        rec,
        "12640582353013470300",
        Some(&hex!("5c747069ad5e6caf")),
        12640582353013470300u64,
    );
    test_compile_ser(
        rec,
        "10476150832182888270",
        Some(&hex!("4e87ac45adc36291")),
        10476150832182888270u64,
    );
    test_compile_ser(
        rec,
        "1634336197892175643",
        Some(&hex!("1bf7ece61c54ae16")),
        1634336197892175643u64,
    );
    test_compile_ser(
        rec,
        "8054772126963294323",
        Some(&hex!("73605dce924cc86f")),
        8054772126963294323u64,
    );
    test_compile_ser(
        rec,
        "15785621808981151287",
        Some(&hex!("370efb4266cb11db")),
        15785621808981151287u64,
    );
    test_compile_ser(
        rec,
        "5969916763410203727",
        Some(&hex!("4f04b795aa67d952")),
        5969916763410203727u64,
    );
    test_compile_ser(
        rec,
        "8744613680266444",
        Some(&hex!("cc3c1c082e111f00")),
        8744613680266444u64,
    );
    test_compile_ser(
        rec,
        "13518752403636168634",
        Some(&hex!("ba273189af419cbb")),
        13518752403636168634u64,
    );
    test_compile_ser(
        rec,
        "17063061512885227165",
        Some(&hex!("9dce29c00a2cccec")),
        17063061512885227165u64,
    );
}
