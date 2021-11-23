use hex_literal::hex;
mod common;
use common::{test_compile_ser, test_load_from_string};

#[test]
fn test_tuple() {
    let rec = "(i32)";
    test_compile_ser(rec, "(0)", Some(&hex!("00000000")), (0i32,));
    test_compile_ser(rec, "(1)", Some(&hex!("01000000")), (1i32,));
    test_compile_ser(
        rec,
        "(-1997293040)",
        Some(&hex!("10baf388")),
        (-1997293040i32,),
    );
    test_compile_ser(rec, "(150517066)", Some(&hex!("4ab5f808")), (150517066i32,));

    let rec = "(bool, u32)";
    test_compile_ser(
        rec,
        "(false, 3725879192)",
        Some(&hex!("00986714de")),
        (false, 3725879192u32),
    );
    test_compile_ser(
        rec,
        "(true, 3017113990)",
        Some(&hex!("018681d5b3")),
        (true, 3017113990u32),
    );

    // Tuple recipe implementation works up to 12 elements
    test_load_from_string("(0)", (0i32,));
    test_load_from_string("(0, 1)", (0i32, 1i32));
    test_load_from_string("(0, 1, 2)", (0i32, 1i32, 2i32));
    test_load_from_string("(0, 1, 2, 3)", (0i32, 1i32, 2i32, 3i32));
    test_load_from_string("(0, 1, 2, 3, 4)", (0i32, 1i32, 2i32, 3i32, 4i32));
    test_load_from_string("(0, 1, 2, 3, 4, 5)", (0i32, 1i32, 2i32, 3i32, 4i32, 5i32));
    test_load_from_string(
        "(0, 1, 2, 3, 4, 5, 6)",
        (0i32, 1i32, 2i32, 3i32, 4i32, 5i32, 6i32),
    );
    test_load_from_string(
        "(0, 1, 2, 3, 4, 5, 6, 7)",
        (0i32, 1i32, 2i32, 3i32, 4i32, 5i32, 6i32, 7i32),
    );
    test_load_from_string(
        "(0, 1, 2, 3, 4, 5, 6, 7, 8)",
        (0i32, 1i32, 2i32, 3i32, 4i32, 5i32, 6i32, 7i32, 8i32),
    );
    test_load_from_string(
        "(0, 1, 2, 3, 4, 5, 6, 7, 8, 9)",
        (0i32, 1i32, 2i32, 3i32, 4i32, 5i32, 6i32, 7i32, 8i32, 9i32),
    );
    test_load_from_string(
        "(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10)",
        (
            0i32, 1i32, 2i32, 3i32, 4i32, 5i32, 6i32, 7i32, 8i32, 9i32, 10i32,
        ),
    );
    test_load_from_string(
        "(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11)",
        (
            0i32, 1i32, 2i32, 3i32, 4i32, 5i32, 6i32, 7i32, 8i32, 9i32, 10i32, 11i32,
        ),
    );

    test_load_from_string("(99, false, (-1, 4))", (99, false, (-1, 4)));
}
