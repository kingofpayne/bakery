//! This crate allows loading text data files as serialized binary objects.
//!
//! Data files can have a syntax similar to rust data definition, and must follow a recipe (or
//! schema) defined either in a recipe file or built in the program using the `Recipe` trait.
//!
//! When a data file is parsed, a binary representation is built and cached into a binary file, and
//! this binary representation can then be easily deserialized using, for instance, serde and
//! bincode libraries. If the data file is not modified, loading the data is then very fast. This
//! is very efficient for instance for storing complex asset files in a comprehensible and easily
//! modifiable format. It may also be useful for loading configuration files.
//!
//! # Basic example
//!
//! ```
//! use bakery::load_from_string;
//! use bakery_derive::Recipe;
//! use serde::Deserialize;
//!
//! #[derive(Recipe, Deserialize)]
//! struct GameConfig {
//!     width: u32,
//!     height: u32,
//!     fullscreen: bool
//! }
//!
//! let config: GameConfig = load_from_string("width: 1024, height: 768, fullscreen: true");
//! ```

use num_bigint::{BigInt, Sign};
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest::Span;
use pest_derive::Parser;
use serde::de::DeserializeOwned;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::rc::Rc;

mod tree;
use tree::Tree;

mod recipe;
pub use recipe::Recipe;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct MyParser;

type WriteResult = Result<(), std::io::Error>;

#[derive(Debug)]
pub enum ParseError {
    IncompleteDatParse,
    IncompleteRecParse(usize),
}

enum CompilationError {
    DataNotStruct(u32),
    EnumTypeIsNotInt(u32),
    EnumValueOutOfBounds(u32),
    EnumUndefinedName {
        node_enum: u32,
        node_name: u32,
    },
    EnumUndefinedData {
        data_nid: u32,
    },
    ExpectedDatFloat(u32),
    ExpectedDatInt(u32),
    ExpectedDatStruct(u32),
    ExpectedDatIdentifier(u32),
    GenericArgCountMismatch {
        nid: u32,
        expected: usize,
        current: usize,
    },
    IOError(std::io::Error),
    ParseError(ParseError),
    RedefinedValue(u32),
    TupleSizeMismatch {
        node_tuple: u32,
        node_data: u32,
    },
    UndefinedValue(u32),
    UnresolvedType {
        path: String,
        node: u32,
    },
    ValueOutOfBounds(u32),
}

impl From<ParseError> for CompilationError {
    fn from(e: ParseError) -> Self {
        CompilationError::ParseError(e)
    }
}

#[derive(Debug)]
pub struct SourceLocation {
    source: Rc<String>,
    start: usize,
    end: usize,
}

impl SourceLocation {
    fn new_from_span(source: Rc<String>, span: Span<'_>) -> Self {
        SourceLocation {
            source: source,
            start: span.start(),
            end: span.end(),
        }
    }
}

/// Identifies a type in the node tree
/// Set by string during parsing, then revolved as a node id during compilation
#[derive(Debug, Clone)]
pub enum RecTypeId {
    Path(String),
    Id(u32),
}

impl RecTypeId {
    /// Return the node id or panic if `self` if not a `RecTypeId::Id`
    fn unwrap_id(&self) -> u32 {
        match self {
            Self::Id(value) => *value,
            _ => panic!("unresolved type"),
        }
    }
}

/// Possible elements from recipe and data files
///
/// Values prefixed with Rec refer to recipe definition items. Values prefixed with Dat refer to
/// content data.
#[derive(Debug, Clone)]
pub enum NodeContent {
    /// Integer type
    ///
    /// Nodes of this type are created by the compiler, to populate standard integer types such as
    /// i32, u32, etc.
    RecInt {
        bit_size: u32,
        signed: bool,
    },
    /// Floating point type
    ///
    /// Nodes of this type are created by the compiler, to populate standard f32 and f64 types.
    /// The stored value can be 32 or 64 for respectively f32 and f64.
    RecFloat {
        size: u32,
    },
    /// Generic list type
    RecList,
    /// Generic map type
    RecMap,
    /// Recipe enumeration definition
    ///
    /// `RecEnum` nodes have one `RecEnumItem` child node for each possible enumeration value.
    RecEnum {
        key_type: RecTypeId,
    },
    /// An enumeration possible value
    ///
    /// RecEnumItem can have one child, which can be a RecTuple or RecStruct
    RecEnumItem {
        value: BigInt,
    },
    RecStruct,
    /// Members of structures
    /// Name of the node is the member name in the structure
    /// Type of the structure member is first and unique child
    RecStructMember,
    /// Members of tuples
    /// Name of the node is None
    RecTupleMember {
        tid: RecTypeId,
    },
    /// Node designating another existing type in the tree
    /// Firstly stores the path string to the type, and then is resolved as the pointed type node
    /// Id.
    /// This node can have children, and each RecTypeInst children is a generic type argument to
    /// the parent node.
    RecTypeInst {
        tid: RecTypeId,
    },
    /// Generic type node which may be child of `RecStruct` or native generic types such as
    /// `RecList`.
    /// `index` field corresponds to the index of the generic type in the parent type.
    RecGeneric {
        index: u32,
    },
    DatMap,
    DatTuple,
    DatList,
    /// This node corresponds to a map or structure assignment (we can consider structures as a
    /// subset of maps from a grammar perspective).
    /// Such node have two childen, one for the name or key, and a second for the value.
    DatMapAssignment,
    DatTupleMember,
    RecTuple,
    DatInt {
        repr: String,
    },
    DatFloat {
        repr: String,
    },
    /// Enumeration identifier (works for boolean too)
    /// Enumeration value name stored in node name
    /// Also used for structure assignments.
    DatEnum,
}

impl NodeContent {
    /// Return true if node can have generic type arguments
    fn may_be_generic(&self) -> bool {
        match self {
            NodeContent::RecStruct | NodeContent::RecList | NodeContent::RecMap => true,
            _ => false,
        }
    }
}

/// Node for recipe tree
///
/// Each node can represent a structure definition, a typedef, a namespace...
#[derive(Debug)]
pub struct Node {
    pub name: Option<String>,
    // Native types do not come from the recipe file, so they cannot have any span.
    pub source: Option<SourceLocation>,
    pub content: NodeContent,
}

impl Node {
    /// Create a node with a name but no source.
    ///
    /// # Arguments
    ///
    /// * `name` - Node name
    /// * `content` - Node content
    pub fn new_builtin(name: &str, content: NodeContent) -> Node {
        Node {
            name: Some(name.to_string()),
            source: None,
            content: content,
        }
    }

    /// Create a node with no name and no source.
    ///
    /// # Arguments
    ///
    /// * `content` - Node content
    pub fn new_anonymous(content: NodeContent) -> Node {
        Node {
            name: None,
            source: None,
            content: content,
        }
    }

    fn name_or_anonymous(&self) -> String {
        match &self.name {
            Some(name) => name.clone(),
            None => "?".to_string(),
        }
    }
}

pub type NodeTree = Tree<Node>;

impl NodeTree {
    /// Create a generic type and return created node Id.
    ///
    /// # Arguments
    ///
    /// * `parent` - Parent node Id, or None
    /// * `name` - Name of the type
    /// * `content` - Node content
    /// * `n` - Number of generic types
    fn create_generic_type(
        &mut self,
        parent: Option<u32>,
        name: &str,
        content: NodeContent,
        n: u32,
    ) -> u32 {
        let nid = self.create_with_parent(
            parent,
            Node {
                name: Some(name.to_string()),
                source: None,
                content: content,
            },
        );
        for i in 0..n {
            self.create_with_parent(
                Some(nid),
                Node {
                    name: None,
                    source: None,
                    content: NodeContent::RecGeneric { index: i },
                },
            );
        }
        nid
    }

    /// Create a recipe structure node with no name and no parent
    pub fn create_root_struct(&mut self) -> u32 {
        self.create(Node::new_anonymous(NodeContent::RecStruct))
    }

    /// Create a recipe structure node and return node Id.
    ///
    /// # Arguments
    ///
    /// * `parent` - Parent node
    /// * `name` - Structure node name
    pub fn create_struct(&mut self, parent: Option<u32>, name: &str) -> u32 {
        self.create_with_parent(
            parent,
            Node {
                name: Some(name.to_string()),
                source: None,
                content: NodeContent::RecStruct,
            },
        )
    }

    /// Create a recipe structure member node and return node Id.
    ///
    /// # Arguments
    ///
    /// * `parent` - Parent node
    /// * `name` - Member name
    /// * `nid_type` - Node for the type of the structure
    pub fn create_struct_member(&mut self, parent: u32, name: &str, nid_type: u32) -> u32 {
        let nid = self.create_with_parent(
            Some(parent),
            Node {
                name: Some(name.to_string()),
                source: None,
                content: NodeContent::RecStructMember,
            },
        );
        self.child(nid, nid_type);
        nid
    }

    /// Create a recipe enumeration node and return node Id.
    ///
    /// # Arguments
    ///
    /// * `parent` - Parent node
    /// * `name` - Enumeration name
    pub fn create_enum(&mut self, parent: Option<u32>, name: &str, nid_key_type: u32) -> u32 {
        self.create_with_parent(
            parent,
            Node {
                name: Some(name.to_string()),
                source: None,
                content: NodeContent::RecEnum {
                    key_type: RecTypeId::Id(nid_key_type)
                }
            }
        )
    }

    /// Create an enumeration member node and return node Id.
    ///
    /// # Arguments
    ///
    /// * `parent` - Parent node
    /// * `name` - Enumeration member name
    pub fn create_enum_member(&mut self, parent: u32, name: &str, value: BigInt) -> u32 {
        self.create_with_parent(
            Some(parent),
            Node {
                name:Some(name.to_string()),
                source: None,
                content: NodeContent::RecEnumItem { value: value }
            }
        )
    }

    /// Create a tuple node and return node Id.
    ///
    /// # Arguments
    ///
    /// * `parent` - Parent node
    pub fn create_tuple(&mut self, parent: Option<u32>) -> u32 {
        self.create_with_parent(
            parent,
            Node::new_anonymous( NodeContent::RecTuple )
        )
    }

    /// Create a tuple item node and return node Id.
    ///
    /// # Arguments
    ///
    /// * `parent` - Parent node
    /// * `ty` - Type node
    pub fn create_tuple_member(&mut self, parent: u32, ty: u32) -> u32 {
        self.create_with_parent(
            Some(parent),
            Node::new_anonymous(
                NodeContent::RecTupleMember {
                    tid: RecTypeId::Id(ty)
                }
            )
        )
    }

    /// Parse a type recipe from a string, return recipe node Id.
    ///
    /// # Arguments
    ///
    /// * `rec` - Recipe string
    pub fn parse_recipe_string(&mut self, rec: &str) -> Result<u32, ParseError> {
        let rec = rec.trim_start().trim_end();
        let mut pairs = MyParser::parse(Rule::rec_type_anonymous, rec).unwrap();
        let pair = pairs.next().unwrap();
        let span = pair.as_span();
        assert!(pairs.next().is_none());
        // Check the the whole string has been parsed.
        // pest will match as much as it can, so if there's garbage at the end, it will be ignored.
        // We don't want to ignore this garbage.
        if span.end() != rec.len() {
            return Err(ParseError::IncompleteRecParse(span.end()));
        }
        Ok(self.parse_rec_type(Rc::new(rec.to_string()), pair))
    }

    /// Build recipe from a string, return recipe node Id.
    ///
    /// # Arguments
    ///
    /// * `rec` - Recipe string
    pub fn parse_struct_recipe_string(&mut self, rec: &str) -> Result<u32, ParseError> {
        // Parse recipe
        let rec = rec.trim_start().trim_end();
        let mut pairs = MyParser::parse(Rule::file_rec, rec).unwrap();
        let pair = pairs.next().unwrap();
        let span = pair.as_span();
        assert!(pairs.next().is_none());
        // Check the the whole string has been parsed.
        // pest will match as much as it can, so if there's garbage at the end, it will be ignored.
        // We don't want to ignore this garbage.
        if span.end() != rec.len() {
            return Err(ParseError::IncompleteRecParse(span.end()));
        }

        let nid = self.create(Node {
            name: None,
            source: None,
            content: NodeContent::RecStruct,
        });
        self.parse_rec_struct_declarations(Rc::new(rec.to_string()), pair.into_inner(), nid);
        Ok(nid)
    }

    /// Parse and create a data value node from a string, returning created node Id or compilation
    /// error.
    ///
    /// # Arguments
    ///
    /// * `dat` - Data string
    pub fn parse_dat_value_string(&mut self, dat: &str) -> Result<u32, ParseError> {
        let dat = dat.trim_start().trim_end();
        let mut pairs = MyParser::parse(Rule::dat_value, dat).unwrap();
        let pair = pairs.next().unwrap();
        let span = pair.as_span();
        assert!(pairs.next().is_none());
        // Check parsing completeness
        if span.end() != dat.len() {
            Err(ParseError::IncompleteDatParse)
        } else {
            Ok(self.parse_dat_value(Rc::new(dat.to_string()), pair))
        }
    }

    /// Parse and create a DatMap node from a string, returning created node Id or compilation
    /// error.
    ///
    /// # Arguments
    ///
    /// * `dat` - Data string, struct format without the braces.
    pub fn parse_dat_map_string(&mut self, dat: &str) -> Result<u32, ParseError> {
        let dat = dat.trim_start().trim_end();
        let mut pairs = MyParser::parse(Rule::file_dat, dat).unwrap();
        let pair = pairs.next().unwrap();
        let span = pair.as_span();
        assert!(pairs.next().is_none());
        // Check parsing completeness
        if span.end() != dat.len() {
            Err(ParseError::IncompleteDatParse)
        } else {
            Ok(self.parse_dat_map(Rc::new(dat.to_string()), pair))
        }
    }

    /// Parse and create a type recipe node, returns created node Id.
    ///
    /// # Arguments
    ///
    /// * `pair` - pest parser pair to be read, must be a pair describing a type.
    fn parse_rec_type(&mut self, source: Rc<String>, pair: Pair<Rule>) -> u32 {
        match pair.as_rule() {
            Rule::rec_type_inst => self.parse_rec_type_inst(source, pair),
            Rule::rec_struct => self.parse_rec_struct(source, pair),
            Rule::rec_struct_anonymous => self.parse_rec_struct(source, pair),
            Rule::rec_enum => self.parse_rec_enum(source, pair),
            Rule::rec_enum_anonymous => self.parse_rec_enum(source, pair),
            Rule::rec_tuple => self.parse_rec_tuple(source, pair),
            _ => {
                panic!();
            }
        }
    }

    /// Parse and create a RecTypeInst recipe node, returns created node Id.
    ///
    /// # Arguments
    ///
    /// * `source` - Currently parsed source code
    /// * `pair` - pest parser pair to be read, must be a pair describing a type.
    fn parse_rec_type_inst(&mut self, source: Rc<String>, pair: Pair<Rule>) -> u32 {
        let span = pair.as_span();
        let mut inner = pair.into_inner();
        let nid = self.create(Node {
            name: None,
            source: Some(SourceLocation::new_from_span(source.clone(), span)),
            content: NodeContent::RecTypeInst {
                tid: RecTypeId::Path(inner.next().unwrap().as_str().to_string()),
            },
        });
        // The type instantiation may have generic type arguments
        if let Some(pair) = inner.next() {
            for pair in pair.into_inner() {
                let generic_argument_nid = self.parse_rec_type_inst(source.clone(), pair);
                self.child(nid, generic_argument_nid);
            }
        }
        assert_eq!(inner.next(), None);
        nid
    }

    /// Parse and create a structure recipe node, returns created node Id.
    ///
    /// # Arguments
    ///
    /// * `source` - Currently parsed source code
    /// * `pair` - pest parser pair to be read
    fn parse_rec_struct(&mut self, source: Rc<String>, pair: Pair<Rule>) -> u32 {
        let span = pair.as_span().clone();
        let mut inner = pair.into_inner();
        // Structure can be anonymous or not
        let mut pair = inner.next().unwrap();
        let mut name = None;
        if pair.as_rule() == Rule::identifier {
            name = Some(pair.as_str().to_string());
            pair = inner.next().unwrap();
        };
        let id = self.create(Node {
            name: name,
            source: Some(SourceLocation::new_from_span(source.clone(), span)),
            content: NodeContent::RecStruct,
        });
        if pair.as_rule() == Rule::rec_generic_decl {
            self.parse_rec_generic_decl(source.clone(), id, pair);
            pair = inner.next().unwrap();
        }
        self.parse_rec_struct_declarations(source, pair.into_inner(), id);
        assert!(inner.next().is_none());
        id
    }

    /// Parse a generic types declaration of a structure node.
    /// For each generic type, a `RecGeneric` child node is added to the structure node.
    ///
    /// # Arguments
    ///
    /// * `source` - Currently parsed source code
    /// * `struct_nid` - Structure node Id
    /// * `pair` - pair of rule `Rule::rec_generic_decl` to be parsed
    fn parse_rec_generic_decl(&mut self, source: Rc<String>, struct_nid: u32, pair: Pair<Rule>) {
        let mut arg_index = 0;
        for pair in pair.into_inner() {
            self.create_with_parent(
                Some(struct_nid),
                Node {
                    name: Some(pair.as_str().to_string()),
                    source: Some(SourceLocation::new_from_span(
                        source.clone(),
                        pair.as_span(),
                    )),
                    content: NodeContent::RecGeneric { index: arg_index },
                },
            );
            arg_index = arg_index.checked_add(1).unwrap();
        }
    }

    /// Parse and create a tuple recipe node, returns created node Id.
    ///
    /// # Arguments
    ///
    /// * `source` - Currently parsed source code
    /// * `pair` - pest parser pair to be read
    fn parse_rec_tuple(&mut self, source: Rc<String>, pair: Pair<Rule>) -> u32 {
        // Create the tuple type node
        let tuple_nid = self.create(Node {
            name: None,
            source: Some(SourceLocation::new_from_span(
                source.clone(),
                pair.as_span(),
            )),
            content: NodeContent::RecTuple,
        });
        for pair in pair.into_inner() {
            self.create_with_parent(
                Some(tuple_nid),
                Node {
                    name: None,
                    source: Some(SourceLocation::new_from_span(
                        source.clone(),
                        pair.as_span(),
                    )),
                    content: NodeContent::RecTupleMember {
                        tid: RecTypeId::Path(pair.as_str().to_string()),
                    },
                },
            );
        }
        tuple_nid
    }

    /// Parse the members of a structure recipe node
    ///
    /// This method is separated from parse_rec_struct because it is used to parse a recipe file as
    /// a structure content, without the structure declaration.
    ///
    /// # Arguments
    ///
    /// * `source` - Currently parsed source code
    /// * `inner` - Iterator over the member pairs
    /// * `parent` - Recipe members parent node
    fn parse_rec_struct_declarations(
        &mut self,
        source: Rc<String>,
        inner: Pairs<Rule>,
        parent: u32,
    ) {
        for b in inner {
            let child = self.parse_rec_struct_declaration(source.clone(), b);
            self.child(parent, child);
        }
    }

    /// Parse and create a structure member recipe node, returns created node Id.
    ///
    /// # Arguments
    ///
    /// * `source` - Currently parsed source code
    /// * `pair` - pest parser pair to be read
    fn parse_rec_struct_declaration(&mut self, source: Rc<String>, pair: Pair<Rule>) -> u32 {
        let span = pair.as_span();
        match pair.as_rule() {
            Rule::member => {
                let mut inner = pair.into_inner();
                let struct_nid = self.create(Node {
                    name: Some(inner.next().unwrap().as_str().to_string()),
                    source: Some(SourceLocation::new_from_span(source.clone(), span)),
                    content: NodeContent::RecStructMember,
                });
                // Create one child which is the type of the structure member
                // Child is most of the time a RecTypeId, but it can also be directly a RecTuple,
                // RecStruct, etc.
                let type_pair = inner.next().unwrap();
                let type_nid = self.parse_rec_type(source, type_pair);
                self.child(struct_nid, type_nid);
                struct_nid
            }
            Rule::rec_struct => self.parse_rec_struct(source, pair),
            Rule::rec_enum => self.parse_rec_enum(source, pair),
            _ => {
                panic!()
            }
        }
    }

    /// Parse and create an enum recipe node, returns created node Id.
    ///
    /// # Arguments
    ///
    /// * `source` - Currently parsed source code
    /// * `pair` - pest parser pair to be read
    fn parse_rec_enum(&mut self, source: Rc<String>, pair: Pair<Rule>) -> u32 {
        let span = pair.as_span();
        let mut inner = pair.into_inner();
        let mut pair = inner.next().unwrap();
        let name = if pair.as_rule() == Rule::identifier {
            let res = Some(pair.as_str().to_string());
            pair = inner.next().unwrap();
            res
        } else {
            None
        };
        let enum_nid = self.create(Node {
            name: name,
            source: Some(SourceLocation::new_from_span(source.clone(), span)),
            content: NodeContent::RecEnum {
                key_type: RecTypeId::Path("i32".to_string()),
            },
        });
        // Walk all enumeration values
        for pair in pair.into_inner() {
            let span = pair.as_span();
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let enum_item_nid = self.create_with_parent(
                Some(enum_nid),
                Node {
                    name: Some(name),
                    source: Some(SourceLocation::new_from_span(source.clone(), span.clone())),
                    content: NodeContent::RecEnumItem {
                        value: BigInt::from(0),
                    },
                },
            );
            // Build data type tuple if defined
            if let Some(pair) = inner.next() {
                match pair.as_rule() {
                    Rule::rec_enum_tuple => {
                        let tuple_nid = self.parse_rec_tuple(source.clone(), pair);
                        self.child(enum_item_nid, tuple_nid);
                    }
                    Rule::rec_struct_declarations => {
                        let struct_nid = self.create_with_parent(
                            Some(enum_item_nid),
                            Node {
                                name: None,
                                source: Some(SourceLocation::new_from_span(source.clone(), span)),
                                content: NodeContent::RecStruct,
                            },
                        );
                        self.parse_rec_struct_declarations(
                            source.clone(),
                            pair.into_inner(),
                            struct_nid,
                        );
                    }
                    _ => {
                        panic!()
                    } // Case not allowed by grammar
                }
            };
            assert!(inner.next().is_none());
        }
        assert!(inner.next().is_none());
        enum_nid
    }

    /// Parse and create a DatMap data node, returns created node Id.
    ///
    /// # Arguments
    ///
    /// * `source` - Currently parsed source code
    /// * `pair` - pest parser pair to be read
    fn parse_dat_map(&mut self, source: Rc<String>, pair: Pair<Rule>) -> u32 {
        let span = pair.as_span();
        let node = self.create(Node {
            name: None,
            source: Some(SourceLocation::new_from_span(source.clone(), span)),
            content: NodeContent::DatMap,
        });
        for p in pair.into_inner() {
            let child = self.parse_dat_map_assignment(source.clone(), p);
            self.child(node, child);
        }
        node
    }

    /// Parse and create a DatMapAssigment node, returns created node Id.
    /// Created node will have two children: a first one for the assignment name or key, and a
    /// second one for the value.
    ///
    /// # Arguments
    ///
    /// * `source` - Currently parsed source code
    /// * `pair` - pest parser pair to be read
    fn parse_dat_map_assignment(&mut self, source: Rc<String>, pair: Pair<Rule>) -> u32 {
        let span = pair.as_span();
        let mut inner = pair.into_inner();
        let nid_key = self.parse_dat_value(source.clone(), inner.next().unwrap());
        let nid_value = self.parse_dat_value(source.clone(), inner.next().unwrap());
        let nid = self.create(Node {
            name: None,
            source: Some(SourceLocation::new_from_span(source, span)),
            content: NodeContent::DatMapAssignment,
        });
        assert_eq!(inner.next(), None);
        self.child(nid, nid_key);
        self.child(nid, nid_value);
        nid
    }

    /// Parse and create a tuple data node, returns created node Id.
    ///
    /// # Arguments
    ///
    /// * `source` - Currently parsed source code
    /// * `pair` - pest parser pair to be read
    /// * `content` - Content for the created node
    fn parse_dat_tuple_or_list(
        &mut self,
        source: Rc<String>,
        pair: Pair<Rule>,
        content: NodeContent,
    ) -> u32 {
        if let NodeContent::DatTuple | NodeContent::DatList = content {
        } else {
            panic!();
        }
        let node = self.create(Node {
            name: None,
            source: Some(SourceLocation::new_from_span(
                source.clone(),
                pair.as_span(),
            )),
            content: content,
        });
        for p in pair.into_inner() {
            let child = self.parse_dat_value(source.clone(), p);
            self.child(node, child)
        }
        node
    }

    /// Parse and create an enum data node, returns created node Id.
    ///
    /// # Arguments
    ///
    /// * `source` - Currently parsed source code
    /// * `pair` - pest parser pair to be read
    fn parse_dat_enum(&mut self, source: Rc<String>, pair: Pair<Rule>) -> u32 {
        let span = pair.as_span();
        let mut inner = pair.into_inner();
        let node = self.create(Node {
            name: Some(inner.next().unwrap().as_str().to_string()),
            source: Some(SourceLocation::new_from_span(source.clone(), span)),
            content: NodeContent::DatEnum,
        });
        // There might be a following tuple or struct
        if let Some(pair) = inner.next() {
            let child_nid = match pair.as_rule() {
                Rule::dat_tuple => {
                    self.parse_dat_tuple_or_list(source.clone(), pair, NodeContent::DatTuple)
                }
                Rule::dat_map => self.parse_dat_map(source, pair),
                _ => panic!(),
            };
            self.child(node, child_nid);
        }
        node
    }

    /// Parse and create a data primitive value node, returns created node Id.
    ///
    /// # Arguments
    ///
    /// * `source` - Currently parsed source code
    /// * `pair` - pest parser pair to be read
    fn parse_dat_primitive_value(&mut self, source: Rc<String>, pair: Pair<Rule>) -> u32 {
        self.create(Node {
            name: None,
            source: Some(SourceLocation::new_from_span(source, pair.as_span())),
            content: match pair.as_rule() {
                Rule::int => NodeContent::DatInt {
                    repr: pair.as_str().to_string(),
                },
                Rule::float => NodeContent::DatFloat {
                    repr: pair.as_str().to_string(),
                },
                _ => panic!(),
            },
        })
    }

    /// Parse and create a data value node, returns created node Id.
    ///
    /// # Arguments
    ///
    /// * `source` - Currently parsed source code
    /// * `pair` - pest parser pair to be read
    fn parse_dat_value(&mut self, source: Rc<String>, pair: Pair<Rule>) -> u32 {
        match pair.as_rule() {
            Rule::int | Rule::float => self.parse_dat_primitive_value(source, pair),
            Rule::dat_map => self.parse_dat_map(source, pair),
            Rule::dat_enum => self.parse_dat_enum(source, pair),
            Rule::dat_tuple => self.parse_dat_tuple_or_list(source, pair, NodeContent::DatTuple),
            Rule::dat_list => self.parse_dat_tuple_or_list(source, pair, NodeContent::DatList),
            _ => panic!(),
        }
    }

    /// For a given recipe node, returns the number of expected generic type arguments.
    /// If this query is not relevant for a node, 0 is returned.
    fn number_of_generic_types(&self, nid: u32) -> usize {
        let node = self.get_item(nid);
        match node.value.content {
            NodeContent::RecStruct | NodeContent::RecList | NodeContent::RecMap => node
                .children()
                .iter()
                .filter(|&&nid| {
                    if let NodeContent::RecGeneric { .. } = self.get(nid).content {
                        true
                    } else {
                        false
                    }
                })
                .count(),
            _ => 0,
        }
    }

    /// Return lexical path of a given node
    ///
    /// # Arguments
    ///
    /// * `id` - Id of a node
    fn node_path(&self, id: u32) -> String {
        let node = self.get_item(id);
        match *node.parent() {
            Some(parent_id) => {
                match self.get_item(parent_id).parent {
                    // If the parent is root, don't write it in the path for better display for humans.
                    Some(_) => {
                        self.node_path(parent_id)
                            + &"::".to_string()
                            + &node.value.name_or_anonymous()
                    }
                    None => node.value.name_or_anonymous(),
                }
            }
            None => node.value.name_or_anonymous(),
        }
    }

    fn populate_natives(&mut self, node: u32) {
        let natives = [
            (
                "i8",
                NodeContent::RecInt {
                    bit_size: 8,
                    signed: true,
                },
            ),
            (
                "u8",
                NodeContent::RecInt {
                    bit_size: 8,
                    signed: false,
                },
            ),
            (
                "i16",
                NodeContent::RecInt {
                    bit_size: 16,
                    signed: true,
                },
            ),
            (
                "u16",
                NodeContent::RecInt {
                    bit_size: 16,
                    signed: false,
                },
            ),
            (
                "i32",
                NodeContent::RecInt {
                    bit_size: 32,
                    signed: true,
                },
            ),
            (
                "u32",
                NodeContent::RecInt {
                    bit_size: 32,
                    signed: false,
                },
            ),
            (
                "i64",
                NodeContent::RecInt {
                    bit_size: 64,
                    signed: true,
                },
            ),
            (
                "u64",
                NodeContent::RecInt {
                    bit_size: 64,
                    signed: false,
                },
            ),
            ("f32", NodeContent::RecFloat { size: 32 }),
            ("f64", NodeContent::RecFloat { size: 64 }),
        ];

        for native in natives.iter() {
            self.create_with_parent(
                Some(node),
                Node {
                    name: Some(native.0.to_string()),
                    source: None,
                    content: native.1.clone(),
                },
            );
        }

        // Create boolean type using enumeration
        let node_bool_enum = bool::recipe(self);
        self.child(node, node_bool_enum);
        // Create generic types
        self.create_generic_type(Some(node), "List", NodeContent::RecList, 1);
        self.create_generic_type(Some(node), "Map", NodeContent::RecMap, 2);
    }
}

/// Return (min, max) possible values for an integer with given bit size and sign bit specification
///
/// # Arguments
///
/// * `bit_size` - Number of bits encoding the integer
/// * `signed` - Whether the integer includes a sign bit or not
fn int_bounds(bit_size: u32, signed: bool) -> (BigInt, BigInt) {
    let max: BigInt = BigInt::from(2).pow(if signed { bit_size - 1 } else { bit_size }) - 1;
    (
        if signed {
            -max.clone() - 1
        } else {
            BigInt::from(0)
        },
        max,
    )
}

enum WriteIntCheckBoundsError {
    IOError(std::io::Error),
    OutOfBounds,
}

impl From<std::io::Error> for WriteIntCheckBoundsError {
    fn from(e: std::io::Error) -> Self {
        WriteIntCheckBoundsError::IOError(e)
    }
}

/// Write an integer to a stream, or return an error if value is out of bounds.
///
/// # Arguments
///
/// * `wr` - Output stream
/// * `bit_size` - Size in bits of the integer
/// * `signed` - Wheter the integer is signed or not
fn write_int_check_bounds(
    wr: &mut dyn std::io::Write,
    bit_size: u32,
    signed: bool,
    value: &BigInt,
) -> Result<(), WriteIntCheckBoundsError> {
    let (min, max) = int_bounds(bit_size, signed);
    if (*value <= max) && (*value >= min) {
        let bytes = if signed {
            let mut bytes = value.to_signed_bytes_le();
            let negative = value.sign() == Sign::Minus;
            // Extend the sign
            // As we are writing little-endian, we push the bytes after.
            while bytes.len() < (bit_size / 8) as usize {
                bytes.push(if negative { 0xff } else { 0 });
            }
            bytes
        } else {
            let mut bytes = value.to_bytes_le().1;
            while bytes.len() < (bit_size / 8) as usize {
                bytes.push(0);
            }
            bytes
        };
        wr.write(bytes.as_slice())?;
        Ok(())
    } else {
        Err(WriteIntCheckBoundsError::OutOfBounds)
    }
}

struct Compiler<'a> {
    tree: NodeTree,
    io: &'a mut dyn std::io::Write,
    errors: Vec<CompilationError>,
    generic_stack: Vec<Vec<u32>>,
}

impl Compiler<'_> {
    pub fn new<'a>(io: &'a mut dyn std::io::Write) -> Compiler<'a> {
        Compiler {
            tree: NodeTree::new(),
            io: io,
            errors: Vec::new(),
            generic_stack: Vec::new(),
        }
    }

    /// Declare an error
    fn error(&mut self, err: CompilationError) {
        self.errors.push(err);
    }

    /// Walks the tree and resolve all types
    ///
    /// All `RecTypeId` are resolved to their corresponding node Id.
    /// Enumeration values are calculated.
    ///
    /// For generic types instanciation, generic argument count is checked.
    ///
    /// # Arguments
    ///
    /// * `nid` - Node to be resolved. Children nodes are resolved recursively.
    fn resolve_types(&mut self, nid: u32) {
        let node = self.tree.get(nid);
        match node.content.clone() {
            NodeContent::RecInt { .. }
            | NodeContent::RecFloat { .. }
            | NodeContent::RecList
            | NodeContent::RecMap => {}
            NodeContent::RecStruct | NodeContent::RecTuple | NodeContent::RecStructMember => {
                for child_id in self.tree.children(nid).clone() {
                    self.resolve_types(child_id);
                }
            }
            NodeContent::RecTupleMember { tid } => {
                let resolved_tid = self.resolve_type_id(tid.clone(), nid);
                if let NodeContent::RecTupleMember { tid } = &mut self.tree.get_mut(nid).content {
                    *tid = resolved_tid;
                } else {
                    panic!();
                }
            }
            NodeContent::RecEnum { key_type: tid } => {
                let resolved_tid = self.resolve_type_id(tid.clone(), nid);
                if let NodeContent::RecEnum { key_type: tid } = &mut self.tree.get_mut(nid).content
                {
                    *tid = resolved_tid.clone();
                } else {
                    panic!();
                }
                // Enum storage type must be a RecInt
                if let RecTypeId::Id(rec_type_id) = resolved_tid {
                    let rec_type = self.tree.get(rec_type_id);
                    if let NodeContent::RecInt { bit_size, signed } = rec_type.content {
                        // Now calculate the values of the enumeration items
                        let mut next_value = BigInt::from(0);
                        for child_id in self.tree.children(nid).clone() {
                            let child = self.tree.get_mut(child_id);
                            if let NodeContent::RecEnumItem { value } = &mut child.content {
                                let (min, max) = int_bounds(bit_size, signed);
                                if (next_value >= min) && (next_value <= max) {
                                    *value = next_value.clone();
                                    next_value = next_value + 1;
                                } else {
                                    self.error(CompilationError::EnumValueOutOfBounds(child_id));
                                }
                                // Resolve data type if defined
                                if let Some(data_type_node) =
                                    self.tree.get_item(child_id).unique_child_or_none()
                                {
                                    self.resolve_types(data_type_node);
                                }
                            } else {
                                panic!();
                            }
                        }
                    } else {
                        // Type for enum storage is not an integer
                        self.error(CompilationError::EnumTypeIsNotInt(rec_type_id));
                    }
                }
            }
            NodeContent::RecTypeInst { tid } => {
                let resolved_tid = self.resolve_type_id(tid.clone(), nid);
                if let NodeContent::RecTypeInst { tid } = &mut self.tree.get_mut(nid).content {
                    *tid = resolved_tid.clone();
                } else {
                    panic!();
                }
                let children = self.tree.children(nid).clone();
                for child_id in children.clone() {
                    self.resolve_types(child_id);
                }
                if let RecTypeId::Id(resolved_type_nid) = resolved_tid {
                    // Verify that the number of generic argument is equal to the number of generic
                    // types.
                    let expected_arg_count = self.tree.number_of_generic_types(resolved_type_nid);
                    let current_arg_count = children.len();
                    if current_arg_count != expected_arg_count {
                        self.error(CompilationError::GenericArgCountMismatch {
                            nid: nid,
                            expected: expected_arg_count,
                            current: current_arg_count,
                        });
                    }
                }
            }
            NodeContent::RecEnumItem { .. }
            | NodeContent::RecGeneric { .. }
            | NodeContent::DatMap
            | NodeContent::DatTuple
            | NodeContent::DatList
            | NodeContent::DatEnum
            | NodeContent::DatMapAssignment
            | NodeContent::DatTupleMember
            | NodeContent::DatInt { .. }
            | NodeContent::DatFloat { .. } => {}
        }
    }

    /// Resolve a RecTypeId if necessary
    ///
    /// # Arguments
    ///
    /// * `tid` - `RecTypeId` to be resolved
    /// * `nid` - Resolution context node Id
    fn resolve_type_id(&mut self, tid: RecTypeId, nid: u32) -> RecTypeId {
        match &tid {
            RecTypeId::Path(path) => {
                if let Some(id) = self.resolve_typename(nid, &path) {
                    RecTypeId::Id(id)
                } else {
                    self.error(CompilationError::UnresolvedType {
                        path: path.clone(),
                        node: nid,
                    });
                    tid.clone()
                }
            }
            RecTypeId::Id(_) => tid.clone(),
        }
    }

    /// Solve a typename and return corresponding recipe type node Id
    ///
    /// # Arguments
    ///
    /// * `scope` - Current recipe scope node
    /// * `typename` - Searched typename
    fn resolve_typename(&self, scope: u32, typename: &String) -> Option<u32> {
        let scope_node = self.tree.get_item(scope);
        match scope_node.value.content {
            NodeContent::RecStructMember { .. } | NodeContent::RecTupleMember { .. } => {
                match scope_node.parent() {
                    Some(id) => self.resolve_typename(*id, typename),
                    None => None,
                }
            }
            NodeContent::RecStruct => {
                match self
                    .tree
                    .children(scope)
                    .iter()
                    .find(|&&a| self.tree.get(a).name == Some(typename.clone()))
                    .copied()
                {
                    Some(node) => Some(node),
                    // Search in parent
                    None => match scope_node.parent() {
                        Some(id) => self.resolve_typename(*id, typename),
                        None => None,
                    },
                }
            }
            NodeContent::RecTuple
            | NodeContent::RecTypeInst { .. }
            | NodeContent::RecEnumItem { .. } => {
                // Search in parent
                self.resolve_typename(scope_node.parent().unwrap(), typename)
            }
            NodeContent::RecEnum { .. } => match scope_node.parent() {
                Some(id) => self.resolve_typename(*id, typename),
                None => None,
            },
            NodeContent::RecInt { .. }
            | NodeContent::RecFloat { .. }
            | NodeContent::RecList
            | NodeContent::RecMap
            | NodeContent::RecGeneric { .. } => None,
            NodeContent::DatMap
            | NodeContent::DatMapAssignment
            | NodeContent::DatTupleMember
            | NodeContent::DatInt { .. }
            | NodeContent::DatFloat { .. }
            | NodeContent::DatEnum
            | NodeContent::DatTuple
            | NodeContent::DatList => {
                panic!()
            }
        }
    }

    /// Write given data node according to given recipe node
    ///
    /// # Arguments
    ///
    /// * `rec_node` - Id of the recipe node
    /// * `dat_node` - Id of the data node
    fn write(&mut self, rec_node: u32, dat_node: u32) -> WriteResult {
        match self.tree.get(rec_node).content.clone() {
            NodeContent::RecInt { bit_size, signed } => {
                self.write_int(rec_node, dat_node, bit_size, signed);
            }
            NodeContent::RecFloat { size } => {
                self.write_float(dat_node, size)?;
            }
            NodeContent::RecList => {
                self.write_list(rec_node, dat_node)?;
            }
            NodeContent::RecMap => {
                self.write_map(rec_node, dat_node)?;
            }
            NodeContent::RecStruct => {
                self.write_struct(rec_node, dat_node)?;
            }
            NodeContent::RecTuple => {
                self.write_tuple(rec_node, dat_node)?
            }
            NodeContent::RecStructMember => {
                let type_nid = self.tree.get_item(rec_node).unique_child();
                self.write_struct_member(rec_node, dat_node, type_nid)?;
            }
            NodeContent::RecEnum { key_type: tid } => {
                self.write_enum(rec_node, dat_node, tid.unwrap_id())?;
            }
            NodeContent::RecTypeInst { tid } => {
                let may_be_generic = self.tree.get(tid.unwrap_id()).content.may_be_generic();
                if may_be_generic {
                    let generics = self.tree.children(rec_node).clone();
                    self.generic_stack.push(generics);
                }
                let write_result = self.write(tid.unwrap_id(), dat_node);
                if may_be_generic {
                    self.generic_stack.pop();
                }
                write_result?;
            }
            NodeContent::RecGeneric { index } => {
                let current_generics = self.generic_stack.last().unwrap().clone();
                let type_nid = current_generics.get(index as usize).unwrap();
                self.write(*type_nid, dat_node)?;
            }
            // RecEnumItem written during write_enum, so this case cannot happen
            NodeContent::RecEnumItem { .. }
            // RecTupleMember written during write_tuple, so this case cannot happen
            | NodeContent::RecTupleMember { .. }
            | NodeContent::DatMap
            | NodeContent::DatMapAssignment
            | NodeContent::DatTupleMember
            | NodeContent::DatInt { .. }
            | NodeContent::DatFloat { .. }
            | NodeContent::DatEnum
            | NodeContent::DatTuple
            | NodeContent::DatList => {
                panic!();
            }
        }
        Ok(())
    }

    /// Write given data node as a native integer
    ///
    /// # Arguments
    ///
    /// * `_rec_nid` - Recipe node Id, unused.
    /// * `dat_nid` - Data node Id
    /// * `bit_size` - Integer bit size
    /// * `signed` - Wether the integer has sign bit or not
    fn write_int(&mut self, _rec_nid: u32, dat_nid: u32, bit_size: u32, signed: bool) {
        if let NodeContent::DatInt { repr } = &self.tree.get(dat_nid).content {
            // Convert the value string to an integer.
            // We use a BigInt since the value in the input file can have any number of digits, and
            // we want to be able to check the bounds of this value properly. Using BigInt makes
            // this easy, though this might not be the most fast/optimal way.
            // The following parsing shall never fail if the grammar is correct.
            let int = BigInt::parse_bytes(repr.as_bytes(), 10).unwrap();
            match write_int_check_bounds(&mut self.io, bit_size, signed, &int) {
                Ok(_) => {}
                Err(_) => {
                    self.error(CompilationError::ValueOutOfBounds(dat_nid));
                }
            }
        } else {
            self.error(CompilationError::ExpectedDatInt(dat_nid));
        }
    }

    /// Write given data node as a floating point number
    ///
    /// # Arguments
    ///
    /// * `_rec_nid` - Recipe node Id
    /// * `dat_nid` - Data node Id
    /// * `bit_size` - 32 for f32, 64 for f64
    fn write_float(&mut self, dat_nid: u32, bit_size: u32) -> WriteResult {
        // Data node can be either DatFloat or DatInt.
        if let NodeContent::DatInt { repr } | NodeContent::DatFloat { repr } =
            &self.tree.get(dat_nid).content
        {
            match bit_size {
                32 => {
                    if let Ok(f) = repr.parse::<f32>() {
                        self.io.write(&f.to_le_bytes())?;
                        Ok(())
                    } else {
                        panic!();
                    }
                }
                64 => {
                    if let Ok(f) = repr.parse::<f64>() {
                        self.io.write(&f.to_le_bytes())?;
                        Ok(())
                    } else {
                        panic!();
                    }
                }
                _ => {
                    panic!();
                }
            }
        } else {
            self.error(CompilationError::ExpectedDatFloat(dat_nid));
            Ok(())
        }
    }

    /// Write given data node as given List node
    ///
    /// # Arguments
    ///
    /// * `rec_nid` - Id of the recipe structure node
    /// * `dat_nid` - Id of the data node
    fn write_list(&mut self, rec_nid: u32, dat_nid: u32) -> WriteResult {
        let item_type_nid = self.tree.unique_child(rec_nid);
        let items = self.tree.children(dat_nid).clone();
        let bytes = items.len().to_le_bytes();
        self.io.write(&bytes)?;
        //ctx.io.write(&items.len().to_le_bytes())?;  // Write size of list
        for &item_nid in items.iter() {
            self.write(item_type_nid, item_nid)?
        }
        Ok(())
    }

    /// Write given data node as given Map node
    ///
    /// # Arguments
    ///
    /// * `rec_nid` - Id of the recipe structure node
    /// * `dat_nid` - Id of the data node
    fn write_map(&mut self, rec_nid: u32, dat_nid: u32) -> WriteResult {
        let generic_args = self.tree.children(rec_nid).clone();
        assert_eq!(generic_args.len(), 2);
        let items = self.tree.children(dat_nid).clone();
        let bytes = items.len().to_le_bytes();
        self.io.write(&bytes)?;
        for &item_nid in items.iter() {
            let item_children = self.tree.children(item_nid).clone();
            assert_eq!(item_children.len(), 2);
            for i in 0..2 {
                self.write(generic_args[i], item_children[i])?
            }
        }
        Ok(())
    }

    /// Write given data node as given struct recipe node
    ///
    /// # Arguments
    ///
    /// * `rec_node` - Id of the recipe structure node
    /// * `dat_node` - Id of the data node
    fn write_struct(&mut self, rec_node: u32, dat_node: u32) -> WriteResult {
        // Check dat_node is a structure
        let dat_node_item = self.tree.get_item(dat_node);
        if let NodeContent::DatMap = dat_node_item.value.content {
            // Parser cannot distinguish between structures and maps without the recipe context, so
            // we must check that all the children have an identifier for the key and not something
            // else.
            let mut error = false;
            for nid_child in dat_node_item.children().clone() {
                let child = self.tree.get_item(nid_child);
                if let NodeContent::DatMapAssignment = child.value.content {
                    // We must check that there is no associated enumeration data, so we can be
                    // sure this is only an identifier.
                    let children = child.children();
                    assert_eq!(children.len(), 2);
                    let key = self.tree.get_item(children[0]);
                    if key.children().len() != 0 {
                        self.error(CompilationError::ExpectedDatIdentifier(nid_child));
                        error = true;
                    }
                } else {
                    self.error(CompilationError::ExpectedDatIdentifier(nid_child));
                    error = true;
                }
            }

            // Don't continue if previous check failed.
            if error {
                return Ok(());
            }

            // Iterate all members of the recipe structure
            for &child in self.tree.children(rec_node).clone().iter() {
                match self.tree.get(child).content {
                    NodeContent::RecStructMember { .. } => self.write(child, dat_node)?,
                    NodeContent::RecEnum { .. }
                    | NodeContent::RecGeneric { .. }
                    | NodeContent::RecStruct { .. }
                    | NodeContent::RecTuple
                    | NodeContent::RecInt { .. }
                    | NodeContent::RecFloat { .. }
                    | NodeContent::RecList
                    | NodeContent::RecMap
                    | NodeContent::RecTypeInst { .. } => {}
                    NodeContent::RecEnumItem { .. }
                    | NodeContent::RecTupleMember { .. }
                    | NodeContent::DatMap
                    | NodeContent::DatMapAssignment
                    | NodeContent::DatTupleMember
                    | NodeContent::DatInt { .. }
                    | NodeContent::DatFloat { .. }
                    | NodeContent::DatEnum
                    | NodeContent::DatTuple
                    | NodeContent::DatList => {
                        panic!()
                    }
                }
            }
            Ok(())
        } else {
            self.errors.push(CompilationError::DataNotStruct(dat_node));
            Ok(())
        }
    }

    /// Write given data node as given structure member recipe node
    ///
    /// # Arguments
    ///
    /// * `rec_nid` - Structure member recipe node Id. Node must be a
    ///   `NodeContent::RecStructMember`.
    /// * `dat_nid` - Structure data node Id. One children of this node with the name matching the
    ///   structure member will be written. Node must be a `NodeContent::DatMap`.
    /// * `rec_type_id` - Structure member resolved type Id.
    /// * `typename` - Name of the type of the structure member
    fn write_struct_member(&mut self, rec_nid: u32, dat_nid: u32, rec_type_id: u32) -> WriteResult {
        let dat_node_item = self.tree.get_item(dat_nid);
        if let NodeContent::DatMap = dat_node_item.value.content {
            // Each child of the data node has two children, a first one for the name of the member,
            // and another one for the value.
            // write_struct checks that all children have an identifier and not data for the key.
            // We must find a unique data member of the given member name
            let name = self.tree.get(rec_nid).name.clone();
            let dat_node_children = dat_node_item.children.clone();
            let candidates: Vec<u32> = dat_node_children
                .iter()
                .filter(|&&a| {
                    let children = self.tree.get_item(a).children();
                    assert_eq!(children.len(), 2);
                    self.tree.get(children[0]).name == name
                })
                .cloned()
                .collect();
            let mut iter = candidates.iter();
            if let Some(member) = iter.next() {
                // The value of the member assignment is the second child of the member node.
                let nid_dat = self.tree.get_item(*member).children[1];
                self.write(rec_type_id, nid_dat)?;
                // The member assignment must be unique. Look if there are some others and
                // report errors.
                while let Some(member) = iter.next() {
                    self.error(CompilationError::RedefinedValue(*member));
                }
            } else {
                self.error(CompilationError::UndefinedValue(rec_nid));
            }
        } else {
            self.error(CompilationError::ExpectedDatStruct(dat_nid));
        }
        Ok(())
    }

    /// Write given data node as given tuple recipe node
    ///
    /// # Arguments
    ///
    /// * `rec_node` - Id of the recipe structure node
    /// * `dat_node` - Id of the data node
    fn write_tuple(&mut self, rec_node: u32, dat_node: u32) -> WriteResult {
        let tuple_children = self.tree.children(rec_node).clone();
        let data_children = self.tree.children(dat_node).clone();
        if tuple_children.len() != data_children.len() {
            self.error(CompilationError::TupleSizeMismatch {
                node_tuple: rec_node,
                node_data: dat_node,
            });
            Ok(())
        } else {
            // All children of `NodeContent::RecTuple` are `NodeContent::RecTupleMember`.
            for (t, d) in tuple_children.iter().zip(data_children.iter()) {
                if let NodeContent::RecTupleMember { tid } = &self.tree.get(*t).content.clone() {
                    self.write(tid.unwrap_id(), *d)?;
                } else {
                    panic!();
                }
            }
            Ok(())
        }
    }

    /// Write given data node as given enumeration recipe node
    ///
    /// If value name does not belong to the enumeration, an error is added in the compilation
    /// context.
    ///
    /// # Arguments
    ///
    /// * `rec_nid` - Id of the structure member recipe node
    /// * `dat_nid` - Id of the structure member data node
    /// * `rec_type_id` - Id of the enumeration storage type node
    fn write_enum(&mut self, rec_nid: u32, dat_nid: u32, rec_type_id: u32) -> WriteResult {
        let dat_node = self.tree.get(dat_nid);
        if let NodeContent::DatEnum = dat_node.content {
            // Find the value corresponding to the name in the enumeration
            match self.tree.children(rec_nid).iter().find(|&&a| {
                self.tree.get(a).name.clone().unwrap() == dat_node.name.clone().unwrap()
            }) {
                Some(nid) => {
                    if let NodeContent::RecInt { bit_size, signed } =
                        self.tree.get(rec_type_id).content
                    {
                        if let NodeContent::RecEnumItem { value } = &self.tree.get(*nid).content {
                            // Bounds are checked but we ignore the result as it MUST pass here.
                            // Enumeration value correctness should be done in a previous
                            // compilation phase. Checking here would produce an error message each
                            // time the enumeration is used, which is not good.
                            match write_int_check_bounds(&mut self.io, bit_size, signed, &value) {
                                Ok(()) => {}
                                Err(WriteIntCheckBoundsError::OutOfBounds) => panic!(), // Cannot happen
                                Err(WriteIntCheckBoundsError::IOError(e)) => return Err(e),
                            }
                            // Write enum value associated data (a DatTuple or DatMap node)
                            if let Some(enum_item_data_type_nid) =
                                self.tree.get_item(*nid).unique_child_or_none()
                            {
                                if let Some(enum_assocoiated_data_nid) =
                                    self.tree.get_item(dat_nid).unique_child_or_none()
                                {
                                    self.write(enum_item_data_type_nid, enum_assocoiated_data_nid)?;
                                } else {
                                    self.error(CompilationError::EnumUndefinedData {
                                        data_nid: dat_nid,
                                    });
                                }
                            }
                            Ok(())
                        } else {
                            panic!();
                        }
                    } else {
                        panic!();
                    }
                }
                None => {
                    // Invalid enumeration value name
                    self.error(CompilationError::EnumUndefinedName {
                        node_enum: rec_nid,
                        node_name: dat_nid,
                    });
                    Ok(())
                }
            }
        } else {
            panic!();
        }
    }

    pub fn compile(&mut self, rec: &str, dat: &str, print_errs: bool) -> Result<(), ()> {
        self.tree.clear();
        let node_root = self.tree.create_root_struct();
        self.tree.populate_natives(node_root); // Add standard types
        match self.tree.parse_recipe_string(rec) {
            Ok(node_rec) => {
                self.tree.child(node_root, node_rec);
                self.resolve_types(node_root);
                // resolve_types may generate errors if some types are unknown. In that case, the tree
                // will have dangling references to types, which is not supported for the rest of the
                // compilation.
                if self.errors.len() == 0 {
                    // Parse data
                    let parsed_dat = match self.tree.get(node_rec).content {
                        NodeContent::RecStruct => self.tree.parse_dat_map_string(dat),
                        _ => self.tree.parse_dat_value_string(dat)
                    };
                    match parsed_dat {
                        Ok(node_dat) => {
                            if let Err(e) = self.write(node_rec, node_dat) {
                                self.error(CompilationError::IOError(e));
                            }
                        }
                        Err(e) => {
                            self.errors.push(e.into());
                        }
                    }
                }
            }
            Err(e) => {
                self.error(e.into());
            }
        }
        if print_errs {
            print_errors(&self.tree, &self.errors);
        }
        if self.errors.len() > 0 {
            return Err(());
        }
        Ok(())
    }
}

fn print_errors(tree: &NodeTree, errors: &Vec<CompilationError>) {
    for error in errors {
        match error {
            CompilationError::DataNotStruct(node) => {
                println!("Error: data {} must be a structure", tree.node_path(*node));
            }
            CompilationError::EnumTypeIsNotInt(node) => {
                println!(
                    "Error: enumeration type {} is not an integer",
                    tree.node_path(*node)
                );
            }
            CompilationError::EnumValueOutOfBounds(node) => {
                println!(
                    "Error: enumeration value {} out of bounds",
                    tree.node_path(*node)
                );
            }
            CompilationError::EnumUndefinedName {
                node_enum,
                node_name,
            } => {
                println!(
                    "Error: invalid name in {} for enumeration {}",
                    tree.node_path(*node_enum),
                    tree.node_path(*node_name)
                );
            }
            CompilationError::EnumUndefinedData { data_nid } => {
                println!(
                    "Error: enumeration data not defined for {}",
                    tree.node_path(*data_nid)
                );
            }
            CompilationError::ExpectedDatFloat(node) => {
                println!("Error: expected float for {}", tree.node_path(*node));
            }
            CompilationError::ExpectedDatInt(node) => {
                println!("Error: expected integer for {}", tree.node_path(*node));
            }
            CompilationError::ExpectedDatStruct(node) => {
                println!(
                    "Error: expected structure for {}={}",
                    *node,
                    tree.node_path(*node)
                );
            }
            CompilationError::ExpectedDatIdentifier(nid) => {
                println!(
                    "Error: expected identifier for structure assignement at {}",
                    tree.node_path(*nid)
                );
            }
            CompilationError::GenericArgCountMismatch {
                nid,
                expected,
                current,
            } => {
                println!(
                    "Error: invalid generic type argument count for {}, expected {}, got {}",
                    tree.node_path(*nid),
                    expected,
                    current
                );
            }
            CompilationError::ParseError(e) => match e {
                ParseError::IncompleteDatParse => println!("Error: incomplete data parse"),
                ParseError::IncompleteRecParse(nid) => {
                    println!("Error: incomplete recipe parse, parsed {} characters", nid)
                }
            },
            CompilationError::IOError(e) => {
                println!("Error: {}", e);
            }
            CompilationError::RedefinedValue(node) => {
                println!("Error: {} already defined", tree.node_path(*node));
            }
            CompilationError::TupleSizeMismatch {
                node_tuple,
                node_data,
            } => {
                println!(
                    "Error: incorrect number of elements in {} for tuple {}",
                    tree.node_path(*node_data),
                    tree.node_path(*node_tuple)
                );
            }
            CompilationError::UndefinedValue(node) => {
                println!("Error: {} is undefined", tree.node_path(*node));
            }
            CompilationError::UnresolvedType { path, node } => {
                println!(
                    "Error: unresolved typename \"{}\" for {}",
                    path,
                    tree.node_path(*node)
                );
            }
            CompilationError::ValueOutOfBounds(node) => {
                println!("Error: value {} out of bounds", tree.node_path(*node));
            }
        }
    }
}

#[derive(Debug)]
pub enum LoadError {
    CompilationErrors,
    InvalidRecExtension,
    InvalidDatExtension,
    RecFileAccess,
    DatFileAccess,
}

/// Checks if a binary file needs to be compiled, by looking if the binary file exists and if its
/// modification date is older than the recipe and data files.
///
/// # Arguments
///
/// * `rec_path` - Path to the recipe file, or None if recipe is built-in
/// * `dat_path` - Path to the data file
/// * `bin_path` - Path to the binary file
fn is_compilation_required(
    rec_path: Option<&str>,
    dat_path: &str,
    bin_path: &str,
) -> Result<bool, LoadError> {
    match std::fs::metadata(bin_path) {
        Ok(bin_metadata) => match std::fs::metadata(dat_path) {
            Ok(dat_metadata) => {
                let bin_modified = bin_metadata.modified().unwrap();
                if let Some(rec_path) = rec_path {
                    match std::fs::metadata(rec_path) {
                        Ok(rec_metadata) => Ok((bin_modified > dat_metadata.modified().unwrap())
                            || (bin_modified > rec_metadata.modified().unwrap())),
                        Err(_) => Err(LoadError::RecFileAccess),
                    }
                } else {
                    Ok(bin_modified > dat_metadata.modified().unwrap())
                }
            }
            Err(_) => Err(LoadError::DatFileAccess),
        },
        // binary file does not exist, we must compile
        Err(_) => Ok(true),
    }
}

/// Write the binary representation of string data to be compiled, with the recipe given as a
/// string.
///
/// # Arguments
///
/// * `dest` - A writable stream
/// * `rec` - Recipe string
/// * `dat` - Data string
pub fn write_from_string_with_recipe<'a>(out: &'a mut dyn std::io::Write, rec: &str, dat: &str)
    -> Result<(), LoadError>
{
    let mut compiler = Compiler::new(out);
    if let Err(()) = compiler.compile(rec, dat, true) {
        return Err(LoadError::CompilationErrors)
    } else {
        Ok(())
    }
}

/// Load an object from a data file, with recipe built using Recipe trait.
///
/// If the binary image of the data file does not exist or is deprecated, it is built and cached.
/// If the recipe changes because the program code has been modified, binary must be removed
/// manually as it is currently not detected.
///
/// # Arguments
///
/// * `dat_path` - Path to the data file. File extension must be .dat
pub fn load_from_file<T>(path: &str) -> Result<T, LoadError>
where
    T: DeserializeOwned + Recipe,
{
    // Check extension is correct
    let path = Path::new(path);
    if path.extension() != Some(OsStr::new("dat")) {
        return Err(LoadError::InvalidDatExtension);
    }
    let path_bin = path.with_extension("bin");
    if is_compilation_required(None, path.to_str().unwrap(), path_bin.to_str().unwrap())? {
        let mut dat = String::new();
        File::open(path).unwrap().read_to_string(&mut dat).unwrap();
        let mut file = File::create(path_bin.clone()).unwrap();
        let mut compiler = Compiler::new(&mut file);
        let nid_rec = T::recipe(&mut compiler.tree);
        let nid_dat = compiler.tree.parse_dat_map_string(dat.as_str()).unwrap();
        compiler.resolve_types(nid_rec);
        compiler.write(nid_rec, nid_dat).unwrap();
        // TODO: in this case, we should build in RAM, save to file and deserialize from RAM, that
        // should be faster.
    }
    let file = File::open(path_bin).unwrap();
    return Ok(bincode::deserialize_from(file).unwrap());
}

/// Load data from a string, with recipe built using [`Recipe`] trait.
///
/// # Arguments
///
/// * `dat` - Data string
///
/// # Example
///
/// This example shows how to load a structure from a string:
/// ```
/// use bakery::load_from_string;
/// use bakery_derive::Recipe;
/// use serde::Deserialize;
///
/// #[derive(Recipe, Deserialize, Debug, PartialEq)]
/// struct GameConfig {
///     width: u32,
///     height: u32,
///     fullscreen: bool
/// }
///
/// let config: GameConfig = load_from_string("width: 1024, height: 768, fullscreen: true");
/// assert_eq!(config, GameConfig { width: 1024, height: 768, fullscreen: true });
/// ```
///
/// This example shows how to load a list from a string. Note that the [`Recipe`] trait for [`Vec`] is
/// implemented by the library.
/// ```
/// use bakery::load_from_string;
///
/// let values: Vec<i32> = load_from_string("[1, 2, 3]");
/// assert_eq!(values, vec![1, 2, 3]);
/// ```
pub fn load_from_string<T>(dat: &str) -> T
where
    T: Recipe + DeserializeOwned,
{
    let mut bin = Vec::<u8>::new();
    let mut compiler = Compiler::new(&mut bin);
    let nid_rec = T::recipe(&mut compiler.tree);
    let nid_dat = match compiler.tree.get(nid_rec).content {
        NodeContent::RecStruct => compiler.tree.parse_dat_map_string(dat).unwrap(),
        _ => compiler.tree.parse_dat_value_string(dat).unwrap()
    };
    compiler.resolve_types(nid_rec);
    compiler.write(nid_rec, nid_dat).unwrap();
    bincode::deserialize_from(&bin[..]).unwrap()
}

/// Load an object from a data file, given a recipe defined in a recipe file.
///
/// If the binary image of the data file does not exist or is outdated, it is built and cached.
/// If the recipe has been changed, the binary is rebuilt as well.
///
/// # Arguments
///
/// * `rec_path` - Path to the recipe file
/// * `dat_path` - Path to the data file
pub fn load_from_file_with_recipe<T>(
    rec_path: &str,
    dat_path: &str,
    dest: &mut T,
) -> Result<(), LoadError>
where
    T: DeserializeOwned,
{
    // Check extension of recipe and data files are correct
    let rec_path = Path::new(rec_path);
    if rec_path.extension() != Some(OsStr::new("rec")) {
        return Err(LoadError::InvalidRecExtension);
    }
    let dat_path = Path::new(dat_path);
    if dat_path.extension() != Some(OsStr::new("dat")) {
        return Err(LoadError::InvalidDatExtension);
    }
    let bin_path = dat_path.with_extension("bin");

    // If binary file is missing or if it is older than the data or recipe files, rebuild it.
    match is_compilation_required(
        Some(rec_path.to_str().unwrap()),
        dat_path.to_str().unwrap(),
        bin_path.to_str().unwrap(),
    ) {
        Ok(true) => {
            let mut rec = String::new();
            File::open(rec_path)
                .unwrap()
                .read_to_string(&mut rec)
                .unwrap();
            let mut dat = String::new();
            File::open(dat_path)
                .unwrap()
                .read_to_string(&mut dat)
                .unwrap();

            let mut file = File::create(bin_path.clone()).unwrap();
            let mut compiler = Compiler::new(&mut file);
            if let Err(()) = compiler.compile(&rec, &dat, true) {
                return Err(LoadError::CompilationErrors);
            }
        }
        Ok(false) => {}
        Err(e) => {
            return Err(e);
        }
    }

    let file = File::open(bin_path).unwrap();
    *dest = bincode::deserialize_from(file).unwrap();

    Ok(())
}
