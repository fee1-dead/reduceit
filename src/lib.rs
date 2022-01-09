#![forbid(unsafe_code)]
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
use tempfile::Builder;

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
}

impl Node {
    #[inline]
    pub fn new(kind: NodeKind, rule: ReplacementRule, children: Vec<Node>) -> Self {
        Self {
            kind: RefCell::new(kind),
            rule,
            children: RefCell::new(children),
            optional: OptionalStatus::Required,
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

pub enum ReduceRule {
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

    fn reduce_inner(&self, node: &Node) -> io::Result<()> {
        match &node.optional {
            OptionalStatus::Optional => {
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
            OptionalStatus::Required => {}
            _ => {}
        }

        for c in &*node.children.borrow() {
            self.reduce_inner(c)?;
        }

        Ok(())
    }

    pub fn reduce(&self) -> io::Result<()> {
        assert!(self.try_()?);
        self.reduce_inner(&self.root)
    }
}

/// a branch used to reduce a kleene node
pub struct Branch<'a> {
    /// root node of tree
    root: &'a Node,
    /// the kleene node that we are working on.
    kleene: &'a Node,
}

impl Criteria<Node> for Branch<'_> {
    fn passes<'a, I: IntoIterator<Item = &'a Node>>(&mut self, iter: I) -> bool
    where
        Node: 'a,
    {
        false
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
