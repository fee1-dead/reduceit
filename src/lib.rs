#![forbid(unsafe_code)]
pub(crate) mod counting;
pub use counting::TokenCountingVec;

pub mod dd;
pub mod lower;

use std::cell::RefCell;
use std::mem::{self, discriminant};
use std::path::PathBuf;
use std::process::Command;
use std::{fmt, io};

use dd::Criteria;
use proc_macro2::TokenStream;

use smol_str::SmolStr;
use tempfile::{Builder, NamedTempFile};

/// how a node is organized
#[derive(Clone)]
pub enum NodeKind {
    KleeneStar,
    KleenePlus,
    Regular {
        /// tokens, should be non-empty only if there are no children
        /// when printing it will be separated with spaces.
        s: SmolStr,
    },
    Temp(String),
}

impl NodeKind {
    #[inline]
    pub fn regular() -> Self {
        Self::Regular {
            s: SmolStr::new_inline(""),
        }
    }
}

impl Default for NodeKind {
    #[inline]
    fn default() -> Self {
        Self::regular()
    }
}

impl fmt::Debug for NodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KleeneStar => f.debug_tuple("KleeneStar").finish(),
            Self::KleenePlus => f.debug_tuple("KleenePlus").finish(),
            Self::Regular { s } if s.is_empty() => f.debug_tuple("Regular").finish(),
            Self::Regular { s } => f.debug_tuple("Regular").field(s).finish(),
            Self::Temp(s) => f.debug_tuple("Temp").field(s).finish(),
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
    Block,
}

impl ReplacementRule {
    fn replaces(&self, other: &Self) -> bool {
        use ReplacementRule::*;

        let replacer = self;
        let replacee = other;
        match (replacer, replacee) {
            (Exempt, Exempt) => false,
            (a, b) if discriminant(a) == discriminant(b) => true,
            (GenericMethodArg, GenericArg) | (Block, Expr) => true,

            _ => false,
        }
    }
}

#[derive(Debug)]
pub enum OptionalStatus {
    /// node cannot be deleted
    Required,
    /// node can be safely deleted in all circumstances
    Optional,
    /// node can only be deleted when it is the last in a kleene-star/plus
    /// tuple sequence. I.e. the P in Vec<(T, Option<P>)> can only be None
    /// when the tuple is the last element in the `Vec`.
    OptionalWhenTrailing,
}

#[derive(Debug)]
pub struct Node {
    kind: RefCell<NodeKind>,
    /// if a node is optional we say that it is syntactically
    /// allowed to be deleted from the tree.
    optional: OptionalStatus,
    rule: ReplacementRule,
    children: RefCell<Vec<Node>>,
    tokens: usize,
}

impl Node {
    #[inline]
    pub fn new(kind: NodeKind, rule: ReplacementRule, children: TokenCountingVec) -> Self {
        Self {
            kind: RefCell::new(kind),
            rule,
            children: RefCell::new(children.vec),
            tokens: children.tokens,
            optional: OptionalStatus::Required,
        }
    }

    #[inline]
    pub fn simple(children: TokenCountingVec) -> Self {
        Self::new(NodeKind::regular(), ReplacementRule::Exempt, children)
    }

    #[inline]
    pub fn verbatim(ts: TokenStream) -> Self {
        let s = SmolStr::new(ts.to_string());
        Self {
            kind: RefCell::new(NodeKind::Regular { s }),
            optional: OptionalStatus::Optional,
            rule: ReplacementRule::Exempt,
            children: RefCell::default(),
            tokens: 0,
        }
    }

    #[inline]
    pub(crate) fn token(s: SmolStr) -> Self {
        Self {
            kind: RefCell::new(NodeKind::Regular { s }),
            optional: OptionalStatus::Required,
            rule: ReplacementRule::Exempt,
            children: RefCell::default(),
            tokens: 1,
        }
    }
}

pub enum ReduceRule {
    Fn(Box<dyn Fn(NamedTempFile) -> bool>),
    Program(PathBuf),
}

pub struct Reducer {
    pub root: Node,
    pub rule: ReduceRule,
}

impl Reducer {
    /// what we do here is write the file to disk, invoke user-specified checker program,
    /// and wait.
    fn try_(&self) -> io::Result<bool> {
        use io::Write;

        let mut tempfile = Builder::new().prefix("reduced").suffix(".rs").tempfile()?;
        write!(tempfile.as_file_mut(), "{}", &self.root)?;

        match &self.rule {
            ReduceRule::Fn(f) => Ok(f(tempfile)),
            ReduceRule::Program(prog) => {
                let status = Command::new(prog)
                    .current_dir(tempfile.path().parent().unwrap())
                    .arg(tempfile.path().file_name().unwrap())
                    .status()?;

                Ok(status.success())
            }
        }
    }

    fn try_replace_node_with(&self, node: &Node, s: String) -> io::Result<bool> {
        let prev_kind = mem::replace(&mut *node.kind.borrow_mut(), NodeKind::Temp(s));
        let res = self.try_();
        *node.kind.borrow_mut() = prev_kind;
        res
    }

    fn reduce_inner(&self, node: &Node, kleene: bool) -> io::Result<()> {
        if let OptionalStatus::Optional = node.optional {
            // if we can delete the thing..
            if self.try_replace_node_with(node, String::new())? {
                // .. replace it with whitespace. Not empty string though,
                // because it could mess up other tokens by joining them
                *node.kind.borrow_mut() = NodeKind::Regular {
                    s: SmolStr::new(" "),
                };
                // .. and remove its children.
                node.children.borrow_mut().clear();
                // nothing to recurse
                return Ok(());
            }
        }

        let kind = node.kind.borrow();

        match &*kind {
            NodeKind::KleeneStar | NodeKind::KleenePlus => {
                drop(kind);
                // temporarily take children from the node.
                let mut items = mem::take(&mut *node.children.borrow_mut());
                // the branch criteria will replace the kleene node
                // with a temp string containing formatted node. It's children must be empty.
                dd::ddmin(
                    &mut items,
                    &mut Branch {
                        reducer: self,
                        kleene: node,
                    },
                );

                *node.children.borrow_mut() = items;

                for c in &*node.children.borrow() {
                    self.reduce_inner(c, true)?;
                }
            }
            NodeKind::Regular { .. } => {
                drop(kind);
                // we need a bounded breadth first search.

                let oldnoderule = node.rule;

                let mut queue = vec![node.children.borrow()];

                'outer: while !queue.is_empty() {
                    let children = queue.pop().unwrap();

                    for c in &*children {
                        if c.rule.replaces(&oldnoderule) {
                            todo!()
                        }
                    }
                }

                for c in &*node.children.borrow() {
                    self.reduce_inner(c, false)?;
                }
            }
            NodeKind::Temp(_) => unreachable!(),
        }

        Ok(())
    }

    pub fn reduce(&self) -> io::Result<()> {
        assert!(self.try_()?);
        self.reduce_inner(&self.root, false)
    }
}

/// a branch used to reduce a kleene node
pub struct Branch<'a> {
    /// root node of tree
    reducer: &'a Reducer,
    /// the kleene node that we are working on.
    kleene: &'a Node,
}

impl Criteria<Node> for Branch<'_> {
    fn passes<'a, I: IntoIterator<Item = &'a Node>>(&mut self, iter: I) -> bool
    where
        Node: 'a,
    {
        let mut iter = iter.into_iter().peekable();
        if let NodeKind::KleenePlus = &*self.kleene.kind.borrow() {
            // kleene plus does not allow an empty sequence.
            if iter.peek().is_none() {
                return false;
            }
        }

        let prev_kind = mem::replace(
            &mut *self.kleene.kind.borrow_mut(),
            NodeKind::Temp(iter.map(|n| format!("{n} ")).collect()),
        );
        let res = self.reducer.try_().unwrap();
        *self.kleene.kind.borrow_mut() = prev_kind;
        res
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &*self.kind.borrow() {
            NodeKind::Regular { s } if !s.is_empty() => write!(f, "{s} ")?,
            NodeKind::Temp(s) => f.write_str(s)?,
            _ => {}
        }

        for c in &*self.children.borrow() {
            c.fmt(f)?
        }

        Ok(())
    }
}
