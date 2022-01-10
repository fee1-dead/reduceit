use syn::punctuated::Punctuated;

use crate::{tvec, Node, NodeKind, OptionalStatus, ReplacementRule};

use super::Lower;

pub trait LowerKleene: Sized {
    const RULE: ReplacementRule;
    fn lower(self, star: NodeKind) -> Node;

    #[inline]
    fn lower_star(self) -> Node {
        self.lower(NodeKind::KleeneStar)
    }

    #[inline]
    fn lower_plus(self) -> Node {
        self.lower(NodeKind::KleenePlus)
    }
}

impl<T: Lower> LowerKleene for Vec<T> {
    const RULE: ReplacementRule = T::RULE;
    fn lower(self, star: NodeKind) -> Node {
        let kleene = self.into_iter().map(Lower::lower).collect();
        Node::new(star, T::RULE, kleene)
    }
}

impl<T: Lower, P: Lower> LowerKleene for Punctuated<T, P> {
    const RULE: ReplacementRule = T::RULE;

    /// A Punctuated<T, P> is a Kleene-Star/Plus node with children of the tuple
    /// (T, Option<P>). We can't mark `P` as an optional node because it results in
    /// attempts to remove the punctuation where there are items after.
    fn lower(self, star: NodeKind) -> Node {
        let children = self
            .into_pairs()
            .map(|pair| {
                let (val, punct) = pair.into_tuple();

                let mut children = tvec![val.lower()];

                if let Some(punct) = punct {
                    let mut node = punct.lower();
                    node.optional = OptionalStatus::OptionalWhenTrailing;
                    children.push(node);
                }

                Node::simple(children)
            })
            .collect();

        Node::new(star, T::RULE, children)
    }
}
