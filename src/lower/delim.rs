use crate::{tvec, Node, TokenCountingVec};

use smol_str::SmolStr;
use syn::token::{Brace, Bracket, Group, Paren};
use syn::MacroDelimiter;

pub trait LowerDelim {
    fn lower_start(&self) -> TokenCountingVec;
    fn lower_end(&self, children: TokenCountingVec) -> Node;
}

macro_rules! lower_delim_impl {
    ($([$ty:ty][$left:literal $right:literal]),+$(,)?) => {$(
        impl LowerDelim for $ty {
            #[inline]
            fn lower_start(&self) -> TokenCountingVec {
                tvec![Node::token(SmolStr::new_inline($left))]
            }

            #[inline]
            fn lower_end(&self, mut children: TokenCountingVec) -> Node {
                children.push(Node::token(SmolStr::new_inline($right)));
                Node::simple(children)
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
    fn lower_start(&self) -> TokenCountingVec {
        match self {
            Self::Brace(v) => v.lower_start(),
            Self::Bracket(v) => v.lower_start(),
            Self::Paren(v) => v.lower_start(),
        }
    }
    #[inline]
    fn lower_end(&self, tvec: TokenCountingVec) -> Node {
        match self {
            Self::Brace(v) => v.lower_end(tvec),
            Self::Bracket(v) => v.lower_end(tvec),
            Self::Paren(v) => v.lower_end(tvec),
        }
    }
}
