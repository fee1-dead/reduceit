use crate::{OptionalStatus, TokenCountingVec};

use super::Lower;

pub trait LowerOpt {
    fn lower_into(self, list: &mut TokenCountingVec);
}

impl<T: Lower> LowerOpt for Option<T> {
    #[inline]
    fn lower_into(self, list: &mut TokenCountingVec) {
        if let Some(v) = self {
            let mut node = v.lower();
            node.optional = OptionalStatus::Optional;
            list.push(node);
        }
    }
}
