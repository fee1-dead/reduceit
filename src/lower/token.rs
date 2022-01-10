use proc_macro2::Ident;
use quote::ToTokens;
use smol_str::SmolStr;
use syn::token::*;
use syn::{Index, Lifetime, Lit, LitStr};

use crate::{lower::Lower, Node, ReplacementRule};

macro_rules! token_lower_impl {
    (
        $($ty:ty = $lit:literal),+$(,)?
    ) => {$(
        impl Lower for $ty {
            const RULE: ReplacementRule = ReplacementRule::Exempt;
            #[inline]
            fn lower(self) -> Node {
                Node::token(SmolStr::new_inline($lit))
            }
        }
    )+};
}

// these are all tokens known to `syn`. They have no children
token_lower_impl! {
    Abstract = "abstract",
    As = "as",
    Async = "async",
    Auto = "auto",
    Await = "await",
    Become = "become",
    Box = "box",
    Break = "break",
    Const = "const",
    Continue = "continue",
    Crate = "crate",
    Default = "default",
    Do = "do",
    Dyn = "dyn",
    Else = "else",
    Enum = "enum",
    Extern = "extern",
    Final = "final",
    Fn = "fn",
    For = "for",
    If = "if",
    Impl = "impl",
    In = "in",
    Let = "let",
    Loop = "loop",
    Macro = "macro",
    Match = "match",
    Mod = "mod",
    Move = "move",
    Mut = "mut",
    Override = "override",
    Priv = "priv",
    Pub = "pub",
    Ref = "ref",
    Return = "return",
    SelfType = "Self",
    SelfValue = "self",
    Static = "static",
    Struct = "struct",
    Super = "super",
    Trait = "trait",
    Try = "try",
    Type = "type",
    Typeof = "typeof",
    Union = "union",
    Unsafe = "unsafe",
    Unsized = "unsized",
    Use = "use",
    Virtual = "virtual",
    Where = "where",
    While = "while",
    Yield = "yield",
    Add = "+",
    AddEq = "+=",
    And = "&",
    AndAnd = "&&",
    AndEq = "&=",
    At = "@",
    Bang = "!",
    Caret = "^",
    CaretEq = "^=",
    Colon = ":",
    Colon2 = "::",
    Comma = ",",
    Div = "/",
    DivEq = "/=",
    Dollar = "$",
    Dot = ".",
    Dot2 = "..",
    Dot3 = "...",
    DotDotEq = "..=",
    Eq = "=",
    EqEq = "==",
    Ge = ">=",
    Gt = ">",
    Le = "<=",
    Lt = "<",
    MulEq = "*=",
    Ne = "!=",
    Or = "|",
    OrEq = "|=",
    OrOr = "||",
    Pound = "#",
    Question = "?",
    RArrow = "->",
    LArrow = "<-",
    Rem = "%",
    RemEq = "%=",
    FatArrow = "=>",
    Semi = ";",
    Shl = "<<",
    ShlEq = "<<=",
    Shr = ">>",
    ShrEq = ">>=",
    Star = "*",
    Sub = "-",
    SubEq = "-=",
    Tilde = "~",
    Underscore = "_",
}

impl Lower for Ident {
    const RULE: ReplacementRule = ReplacementRule::Exempt;

    #[inline]
    fn lower(self) -> Node {
        let s = self.to_string();
        Node::token(s.into())
    }
}

impl Lower for Lit {
    const RULE: ReplacementRule = ReplacementRule::Exempt;

    #[inline]
    fn lower(self) -> Node {
        let s = self.into_token_stream().to_string();
        Node::token(s.into())
    }
}

impl Lower for LitStr {
    const RULE: ReplacementRule = ReplacementRule::Exempt;

    #[inline]
    fn lower(self) -> Node {
        let s = self.into_token_stream().to_string();
        Node::token(s.into())
    }
}

impl Lower for Lifetime {
    const RULE: ReplacementRule = ReplacementRule::Exempt;

    #[inline]
    fn lower(self) -> Node {
        let s = format!("'{}", self.ident);
        Node::token(s.into())
    }
}

impl Lower for Index {
    const RULE: ReplacementRule = ReplacementRule::Exempt;

    #[inline]
    fn lower(self) -> Node {
        let s = self.index.to_string();
        Node::token(s.into())
    }
}
