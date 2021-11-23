use std::collections::HashMap;

use crate::{Node, NodeContent, NodeTree, RecTypeId};
use num_bigint::BigInt;

// Any type implementing this trait can be directly used as a recipe in the compiler.
pub trait Recipe {
    /// Build recipe node tree of the implemented type, and return created node Id.
    ///
    /// # Arguments
    ///
    /// * `tree` - Current compiler node tree
    fn recipe(tree: &mut NodeTree) -> u32;
}

impl Recipe for u8 {
    fn recipe(tree: &mut NodeTree) -> u32 {
        tree.create(Node::new_builtin(
            "u8",
            NodeContent::RecInt {
                signed: false,
                bit_size: 8,
            },
        ))
    }
}

impl Recipe for i8 {
    fn recipe(tree: &mut NodeTree) -> u32 {
        tree.create(Node::new_builtin(
            "i8",
            NodeContent::RecInt {
                signed: true,
                bit_size: 8,
            },
        ))
    }
}

impl Recipe for u16 {
    fn recipe(tree: &mut NodeTree) -> u32 {
        tree.create(Node::new_builtin(
            "u16",
            NodeContent::RecInt {
                signed: false,
                bit_size: 16,
            },
        ))
    }
}

impl Recipe for i16 {
    fn recipe(tree: &mut NodeTree) -> u32 {
        tree.create(Node::new_builtin(
            "i16",
            NodeContent::RecInt {
                signed: true,
                bit_size: 16,
            },
        ))
    }
}

impl Recipe for u32 {
    fn recipe(tree: &mut NodeTree) -> u32 {
        tree.create(Node::new_builtin(
            "u32",
            NodeContent::RecInt {
                signed: false,
                bit_size: 32,
            },
        ))
    }
}

impl Recipe for i32 {
    fn recipe(tree: &mut NodeTree) -> u32 {
        tree.create(Node::new_builtin(
            "i32",
            NodeContent::RecInt {
                signed: true,
                bit_size: 32,
            },
        ))
    }
}

impl Recipe for u64 {
    fn recipe(tree: &mut NodeTree) -> u32 {
        tree.create(Node::new_builtin(
            "u64",
            NodeContent::RecInt {
                signed: false,
                bit_size: 64,
            },
        ))
    }
}

impl Recipe for i64 {
    fn recipe(tree: &mut NodeTree) -> u32 {
        tree.create(Node::new_builtin(
            "i64",
            NodeContent::RecInt {
                signed: true,
                bit_size: 64,
            },
        ))
    }
}

impl Recipe for f32 {
    fn recipe(tree: &mut NodeTree) -> u32 {
        tree.create(Node::new_builtin("f32", NodeContent::RecFloat { size: 32 }))
    }
}

impl Recipe for f64 {
    fn recipe(tree: &mut NodeTree) -> u32 {
        tree.create(Node::new_builtin("f32", NodeContent::RecFloat { size: 64 }))
    }
}

impl Recipe for bool {
    fn recipe(tree: &mut NodeTree) -> u32 {
        let node_u8 = u8::recipe(tree);

        let node_bool = tree.create(Node::new_builtin(
            "bool",
            NodeContent::RecEnum {
                key_type: RecTypeId::Id(node_u8),
            },
        ));

        tree.create_with_parent(
            Some(node_bool),
            Node::new_builtin(
                "false",
                NodeContent::RecEnumItem {
                    value: BigInt::from(0),
                },
            ),
        );

        tree.create_with_parent(
            Some(node_bool),
            Node::new_builtin(
                "true",
                NodeContent::RecEnumItem {
                    value: BigInt::from(1),
                },
            ),
        );

        node_bool
    }
}

impl<T: Recipe> Recipe for Vec<T> {
    fn recipe(tree: &mut NodeTree) -> u32 {
        let t = T::recipe(tree);
        let nid = tree.create(Node::new_anonymous(NodeContent::RecList));
        tree.child(nid, t);
        nid
    }
}

impl<K: Recipe, T: Recipe> Recipe for HashMap<K, T> {
    fn recipe(tree: &mut NodeTree) -> u32 {
        let node_k = K::recipe(tree);
        let node_t = T::recipe(tree);
        let node = tree.create(Node::new_anonymous(NodeContent::RecMap));
        tree.child(node, node_k);
        tree.child(node, node_t);
        node
    }
}

macro_rules! tuple_impls {
    ($(
        $Tuple:ident {
            $(($idx:tt) -> $T:ident)+
        }
    )+) => {
        $(
            impl <$($T:Recipe),+> Recipe for ($($T,)+) {
                fn recipe(tree: &mut NodeTree) -> u32 {
                    let nid = tree.create(Node::new_anonymous(NodeContent::RecTuple));
                    $(
                        let t = $T::recipe(tree);
                        tree.create_with_parent(
                            Some(nid),
                            Node::new_anonymous(NodeContent::RecTupleMember { tid: RecTypeId::Id(t) })
                        );
                    )+
                    nid
                }
            }
        )+
    }
}

tuple_impls! {
    Tuple1 {
        (0) -> A
    }
    Tuple2 {
        (0) -> A
        (1) -> B
    }
    Tuple3 {
        (0) -> A
        (1) -> B
        (2) -> C
    }
    Tuple4 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
    }
    Tuple5 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
    }
    Tuple6 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
    }
    Tuple7 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
    }
    Tuple8 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
        (7) -> H
    }
    Tuple9 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
        (7) -> H
        (8) -> I
    }
    Tuple10 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
        (7) -> H
        (8) -> I
        (9) -> J
    }
    Tuple11 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
        (7) -> H
        (8) -> I
        (9) -> J
        (10) -> K
    }
    Tuple12 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
        (7) -> H
        (8) -> I
        (9) -> J
        (10) -> K
        (11) -> L
    }
}
