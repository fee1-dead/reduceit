use crate::{Node, OptionalStatus};

use super::Lower;

pub trait LowerOpt {
    fn lower_into(self, list: &mut Vec<Node>);
}

impl<T: Lower> LowerOpt for Option<T> {
    #[inline]
    fn lower_into(self, list: &mut Vec<Node>) {
        if let Some(v) = self {
            let mut node = v.lower();
            node.optional = OptionalStatus::Optional;
            list.push(node);
        }
    }
}
