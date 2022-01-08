use syn::punctuated::{Pair, Punctuated};

use crate::{Node, NodeKind, ReplacementRule};

use super::{Lower, LowerOpt};

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
    fn lower(self, star: NodeKind) -> Node {
        let mut opt_trailing_seg = None;
        let mut kleene = Vec::with_capacity(self.len());
        for pair in self.into_pairs() {
            match pair {
                Pair::Punctuated(v, p) => {
                    let vnode = v.lower();
                    let pnode = p.lower();
                    let tuple = Node::new(
                        NodeKind::regular(),
                        ReplacementRule::Exempt,
                        vec![vnode, pnode],
                    );
                    kleene.push(tuple);
                }
                Pair::End(p) => opt_trailing_seg = Some(p),
            }
        }

        let mut children = vec![Node::new(star, T::RULE, kleene)];
        opt_trailing_seg.lower_into(&mut children);

        Node::new(NodeKind::regular(), ReplacementRule::Exempt, children)
    }
}
