mod ast;
mod kleene;
mod token;
pub use kleene::LowerKleene;

mod delim;
pub use delim::LowerDelim;

mod opt;
pub use opt::LowerOpt;
use proc_macro2::TokenStream;

use crate::{Node, ReplacementRule};

pub trait Lower {
    const RULE: ReplacementRule;
    fn lower(self) -> Node;
}

impl<T: Lower> Lower for Box<T> {
    const RULE: ReplacementRule = T::RULE;
    #[inline]
    fn lower(self) -> Node {
        (*self).lower()
    }
}

impl Lower for TokenStream {
    const RULE: ReplacementRule = ReplacementRule::Exempt;
    #[inline]
    fn lower(self) -> Node {
        Node::verbatim(Self::RULE, self)
    }
}
