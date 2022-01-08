#![forbid(clippy::undocumented_unsafe_blocks)]

use std::cell::RefCell;
use std::error::Error;
use std::mem::discriminant;

use proc_macro2::TokenStream;

use smol_str::SmolStr;




pub mod lower;

/// how a node is organized
pub enum NodeKind {
    KleeneStar,
    KleenePlus,
    Opt,
    Regular {
        /// tokens, should be non-empty only if there are no children
        /// when printing it will be separated with spaces.
        s: SmolStr,
    },
}

impl NodeKind {
    #[inline]
    pub fn regular() -> Self {
        Self::Regular {
            s: SmolStr::new_inline(""),
        }
    }
}

/// basically we want to know whether one node can replace another
#[derive(Clone, Copy, Debug)]
pub enum ReplacementRule {
    /// exempt from replacement
    Exempt,
    /// attr
    Attribute,
    /// path segment generic arguments
    PathArgs,
    /// a type
    Type,
    /// an expression
    Expr,
    Stmt,
    Path,
    Meta,
    Pat,
    Item,
    Local,
    Arm,
    GenericMethodArg,
    GenericArg,
    QPath,
    Use,
}

impl ReplacementRule {
    fn replaces(&self, other: &Self) -> bool {
        use ReplacementRule::*;

        match (self, other) {
            (Exempt, Exempt) => false,
            (a, b) if discriminant(a) == discriminant(b) => true,
            (GenericMethodArg, GenericArg) => true,
            _ => false,
        }
    }
}

pub struct NodeInner {
    kind: NodeKind,
    rule: ReplacementRule,
    children: Vec<Node>,
}

pub struct Node {
    inner: RefCell<NodeInner>,
}

impl Node {
    #[inline]
    pub fn new(kind: NodeKind, rule: ReplacementRule, children: Vec<Node>) -> Self {
        let inner = NodeInner {
            kind,
            rule,
            children,
        };
        Self {
            inner: RefCell::new(inner),
        }
    }

    #[inline]
    pub fn simple(children: Vec<Node>) -> Self {
        Self::new(NodeKind::regular(), ReplacementRule::Exempt, children)
    }

    #[inline]
    pub fn verbatim(rule: ReplacementRule, ts: TokenStream) -> Self {
        let s = SmolStr::new(ts.to_string());
        Self::new(NodeKind::Regular { s }, rule, vec![])
    }
}

fn main() -> Result<(), std::boxed::Box<dyn Error>> {
    Ok(())
}
