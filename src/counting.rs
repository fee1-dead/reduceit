use crate::Node;

pub(crate) const fn count_helper<const N: usize>(_: [(); N]) -> usize {
    N
}

#[macro_export]
macro_rules! tvec {
    [@replace_expr($exp:expr)] => (());
    [@count_exprs($($elems:expr),*)] => (crate::counting::count_helper([ $( crate::tvec!(@replace_expr($elems)), )* ]));
    [] => (crate::TokenCountingVec::default());
    [$($elem:expr),+$(,)?] => {{
        const CNT: usize = crate::tvec!(@count_exprs($($elem),*));

        let mut vec = crate::TokenCountingVec {
            vec: Vec::with_capacity(CNT),
            tokens: 0,
        };

        $(
            vec.push($elem);
        )+

        vec
    }}
}

#[derive(Default)]
pub struct TokenCountingVec {
    pub vec: Vec<Node>,
    pub tokens: usize,
}

impl TokenCountingVec {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            vec: Vec::with_capacity(capacity),
            tokens: 0,
        }
    }

    pub fn push(&mut self, node: Node) {
        self.tokens += node.tokens;
        self.vec.push(node);
    }
}

impl FromIterator<Node> for TokenCountingVec {
    fn from_iter<T: IntoIterator<Item = Node>>(iter: T) -> Self {
        let mut tokens = 0;
        let vec = iter.into_iter().inspect(|n| tokens += n.tokens).collect();
        Self { tokens, vec }
    }
}
