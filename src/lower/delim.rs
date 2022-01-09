use crate::{Node, NodeKind, ReplacementRule};

use smol_str::SmolStr;
use syn::token::{Brace, Bracket, Group, Paren};
use syn::MacroDelimiter;

pub trait LowerDelim {
    fn lower_start(&self) -> Node;
    fn lower_end(&self, node: &Node);
}

macro_rules! lower_delim_impl {
    ($([$ty:ty][$left:literal $right:literal]),+$(,)?) => {$(
        impl LowerDelim for $ty {
            #[inline]
            fn lower_start(&self) -> Node {
                let left = Node::new(
                    NodeKind::Regular {
                        s: SmolStr::new_inline($left),
                    },
                    ReplacementRule::Exempt,
                    vec![],
                );

                Node::new(NodeKind::regular(), ReplacementRule::Exempt, vec![left])
            }

            #[inline]
            fn lower_end(&self, node: &Node) {
                node.children.borrow_mut().push(
                    Node::new(
                        NodeKind::Regular { s: SmolStr::new_inline($right) },
                        ReplacementRule::Exempt,
                        vec![],
                    )
                )
            }
        }
    )+};
}

lower_delim_impl! {
    [Group]  [""   ""],
    [Brace]  ["{" "}"],
    [Bracket]["[" "]"],
    [Paren]  ["(" ")"],
}

impl LowerDelim for MacroDelimiter {
    #[inline]
    fn lower_start(&self) -> Node {
        match self {
            Self::Brace(v) => v.lower_start(),
            Self::Bracket(v) => v.lower_start(),
            Self::Paren(v) => v.lower_start(),
        }
    }
    #[inline]
    fn lower_end(&self, node: &Node) {
        match self {
            Self::Brace(v) => v.lower_end(node),
            Self::Bracket(v) => v.lower_end(node),
            Self::Paren(v) => v.lower_end(node),
        }
    }
}
