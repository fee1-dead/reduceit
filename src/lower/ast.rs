//! Lowering for syn's AST nodes.
//!
//! This is not called bad code.

use std::cmp;

#[allow(unused_imports)]
use quote::ToTokens;

use syn::punctuated::Pair;
use syn::*;

use crate::{Node, NodeKind, ReplacementRule as R, OptionalStatus};

use super::{Lower, LowerDelim, LowerKleene, LowerOpt};

macro_rules! option_like_lower_impl {
    ($(
        enum $Ty:ident[None = $None:ident] {
            $($Some:ident(_)),*$(,)?
        }
    )*) => {$(
        impl LowerOpt for $Ty {
            #[inline]
            fn lower_into(self, list: &mut Vec<Node>) {
                let mut node = match self {
                    $(Self::$Some(v) => v.lower(),)*
                    Self::$None => return,
                };
                node.optional = OptionalStatus::Optional;
                list.push(node);
            }
        }
    )*};
}

const fn count_helper<const N: usize>(_: [(); N]) -> usize {
    N
}

macro_rules! struct_lower_impl {
    (@replace_expr($($tt:tt)*)) => { () };
    (@count_tts( $( [ $($tt:tt)* ] ),* )) => {{
        const N: usize = count_helper([$( struct_lower_impl!(@replace_expr($($tt)*)), )*]);
        N
    }};
    (@lower( $self:ident $children:ident $field:ident ( $( [$($tt:tt)*] ),* $(,)? ))) => {{
        let n = $self.$field.lower_start();
        {
            let mut nb = n.children.borrow_mut();
            let nchildren = &mut *nb;

            $(struct_lower_impl!(@lower($self nchildren $($tt)*));)*
        }
        $self.$field.lower_end(&n);

        $children.push(n);
    }};
    (@lower( $self:ident $children:ident $field:ident)) => {{
        $children.push($self.$field.lower());
    }};
    (@lower( $self:ident $children:ident $field:ident+)) => {{
        $children.push($self.$field.lower_plus());
    }};
    (@lower( $self:ident $children:ident $field:ident*)) => {{
        $children.push($self.$field.lower_star());
    }};
    (@lower( $self:ident $children:ident $field:ident?)) => {{
        $self.$field.lower_into($children);
    }};
    ($(
        struct $Ty:ty [$rule:ident] {
            $(
                [$($tt:tt)*]
            ),*
            $(,)?
        }
    )*) => {$(
        impl Lower for $Ty {
            const RULE: R = R::$rule;
            fn lower(self) -> Node {
                let mut children = Vec::with_capacity(struct_lower_impl!(@count_tts( $([ $($tt)* ]),* )));
                let c = &mut children;
                $(
                    struct_lower_impl!(@lower( self c $($tt)* ));
                )*
                Node::new(NodeKind::regular(), Self::RULE, children)
            }
        }
    )*};
}

macro_rules! simple_enum_lower_impl {
    ($(
        enum $Ty:ty [$rule:ident $($Verbatim:ident)?]  $([$testexhaustive:ident])? {
            $(
                $Variant:ident
            ),*$(,)?
        }
    )*) => {$(
        impl Lower for $Ty {
            const RULE: R = R::$rule;
            fn lower(self) -> Node {
                let mut node = match self {
                    $(
                        Self::$Variant(v) => v.lower(),
                    )*
                    $(
                        Self::$Verbatim(ts) => Node::verbatim(Self::RULE, ts),
                    )?
                    $(
                        #[cfg(test)]
                        Self::$testexhaustive(_) => unimplemented!(),
                        #[cfg(not(test))]
                        _ => Node::verbatim(Self::RULE, self.into_token_stream())
                    )?
                };
                node.rule = Self::RULE;
                node
            }
        }
    )*};
}

option_like_lower_impl! {
    enum AttrStyle[None = Outer] {
        Inner(_),
    }
    enum TraitBoundModifier[None = None] {
        Maybe(_),
    }
    enum Fields[None = Unit] {
        Named(_), Unnamed(_),
    }
    enum Visibility[None = Inherited] {
        Public(_), Crate(_), Restricted(_),
    }
}

struct_lower_impl! {
    struct File[Exempt] {
        [attrs*], [items*],
    }

    struct Label[Exempt] {
        [name], [colon_token],
    }

    struct Macro[Exempt] {
        [path], [bang_token], [delimiter([tokens])],
    }

    struct MetaList[Exempt] {
        [path],
        [paren_token([nested*])],
    }

    struct MetaNameValue[Exempt] {
        [path],
        [eq_token],
        [lit],
    }

    struct Path[Path] {
        [leading_colon?],
        [segments+],
    }

    struct PathSegment[Exempt] {
        [ident],
        [arguments?],
    }

    struct MethodTurbofish[Exempt] {
        [colon2_token], [lt_token],
        [args*], [gt_token],
    }

    struct Binding[Exempt] {
        [ident], [eq_token], [ty],
    }

    struct LifetimeDef[Exempt] {
        [attrs*], [lifetime], [colon_token?],
        [bounds*],
    }

    struct BoundLifetimes[Exempt] {
        [for_token], [lt_token], [lifetimes*],
        [gt_token],
    }

    struct FieldValue[Exempt] {
        [attrs*], [member], [colon_token?],
        [expr],
    }

    struct Generics[Exempt] {
        [lt_token?], [params*], [gt_token?],
        [where_clause?],
    }

    struct ItemConst[Item] {
        [attrs*], [vis?], [const_token], [ident],
        [colon_token], [ty], [eq_token], [expr],
        [semi_token],
    }

    struct ItemEnum[Item] {
        [attrs*], [vis?], [enum_token], [ident],
        [generics], [brace_token([variants*])],
    }

    struct ItemExternCrate[Item] {
        [attrs*], [vis?], [extern_token],
        [crate_token], [ident], [rename?],
        [semi_token],
    }

    struct ItemFn[Item] {
        [attrs*], [vis?], [sig], [block]
    }

    struct ItemForeignMod[Item] {
        [attrs*], [abi], [brace_token([items*])],
    }

    struct ItemImpl[Item] {
        [attrs*], [defaultness?], [unsafety?],
        [impl_token], [generics], [trait_?],
        [self_ty], [brace_token([items*])],
    }

    struct ItemMacro[Item] {
        [attrs*], [ident?], [mac],
        [semi_token?],
    }

    struct ItemMacro2[Item] {
        [attrs*], [vis?], [macro_token],
        [ident], [rules],
    }

    struct ItemMod[Item] {
        [attrs*], [vis?], [mod_token], [ident],
        [content?], [semi?],
    }

    struct ItemStatic[Item] {
        [attrs*], [vis?], [static_token],
        [mutability?], [ident], [colon_token],
        [ty], [eq_token], [expr], [semi_token],
    }

    struct ItemStruct[Item] {
        [attrs*], [vis?], [struct_token],
        [ident], [generics], [fields?],
        [semi_token?],
    }

    struct ItemTrait[Item] {
        [attrs*], [vis?], [unsafety?], [auto_token?],
        [trait_token], [ident], [generics], [colon_token?],
        [supertraits*], [brace_token([items*])],
    }

    struct ItemTraitAlias[Item] {
        [attrs*], [vis?], [trait_token], [ident],
        [generics], [eq_token], [bounds*], [semi_token],
    }

    struct ItemType[Item] {
        [attrs*], [vis?], [type_token], [ident],
        [generics], [eq_token], [ty], [semi_token],
    }

    struct ItemUnion[Item] {
        [attrs*], [vis?], [union_token], [ident],
        [generics], [fields],
    }

    struct ItemUse[Item] {
        [attrs*], [vis?], [use_token],
        [leading_colon?], [tree],
        [semi_token],
    }

    struct Variant[Exempt] {
        [attrs*], [ident], [fields?],
        [discriminant?],
    }

    struct FieldsNamed[Exempt] {
        [brace_token([named*])]
    }

    struct FieldsUnnamed[Exempt] {
        [paren_token([unnamed*])]
    }

    struct Field[Exempt] {
        [attrs*], [vis?], [ident?],
        [colon_token?], [ty]
    }

    struct Signature[Exempt] {
        [constness?], [asyncness?], [unsafety?],
        [abi?], [fn_token], [ident], [generics],
        [paren_token([inputs*], [variadic?])],
        [output?],
    }

    struct Receiver[Exempt] {
        [attrs*], [reference?], [mutability?],
        [self_token],
    }

    struct Abi[Exempt] {
        [extern_token], [name?],
    }

    struct Variadic[Exempt] {
        [attrs*], [dots],
    }

    struct ForeignItemFn[Exempt] {
        [attrs*], [vis?], [sig], [semi_token]
    }

    struct ForeignItemStatic[Exempt] {
        [attrs*], [vis?], [static_token],
        [mutability?], [ident], [colon_token],
        [ty], [semi_token],
    }

    struct ForeignItemType[Exempt] {
        [attrs*], [vis?], [type_token], [ident],
        [semi_token],
    }

    struct ForeignItemMacro[Exempt] {
        [attrs*], [mac], [semi_token?],
    }

    struct ImplItemConst[Exempt] {
        [attrs*], [vis?], [defaultness?],
        [const_token], [ident], [colon_token],
        [ty], [eq_token], [expr], [semi_token],
    }
    struct ImplItemMethod[Exempt] { // TODO maybe not Exempt
        [attrs*], [vis?], [defaultness?],
        [sig], [block]
    }

    struct ImplItemType[Exempt] {
        [attrs*], [vis?], [defaultness?],
        [ident], [generics], [eq_token], [ty],
        [semi_token],
    }

    struct ImplItemMacro[Use] {
        [attrs*], [mac], [semi_token?],
    }

    struct TraitItemConst[Exempt] {
        [attrs*], [const_token], [ident], [colon_token],
        [ty], [default?], [semi_token]
    }

    struct TraitItemMethod[Exempt] {
        [attrs*], [sig], [default?],
        [semi_token?]
    }

    struct TraitItemType[Exempt] {
        [attrs*], [type_token], [ident], [generics],
        [colon_token?], [bounds*], [default?],
        [semi_token],
    }

    struct TraitItemMacro[Exempt] {
        [attrs*], [mac], [semi_token?],
    }

    struct UsePath[Use] {
        [ident], [colon2_token], [tree],
    }

    struct UseName[Use] { [ident] }

    struct UseRename[Use] {
        [ident], [as_token], [rename],
    }

    struct UseGlob[Use] {
        [star_token]
    }

    struct UseGroup[Use] {
        [brace_token([items*])]
    }

    struct BareFnArg[Exempt] {
        [attrs*], [name?], [ty],
    }

    struct TypeArray[Type] {
        [bracket_token(
            [elem], [semi_token], [len]
        )]
    }

    struct TypeBareFn[Type] {
        [lifetimes?], [unsafety?], [abi?],
        [fn_token], [paren_token([inputs*], [variadic?])],
        [output?]
    }

    struct TypeGroup[Type] {
        [group_token([elem])]
    }

    struct TypeImplTrait[Type] {
        [impl_token], [bounds*]
    }

    struct TypeInfer[Type] { [underscore_token] }

    struct TypeMacro[Type] { [mac] }

    struct TypeNever[Type] { [bang_token] }

    struct TypeParen[Type] {
        [paren_token([elem])]
    }

    struct TypePtr[Type] {
        [const_token?], [mutability?], [elem],
    }

    struct TypeReference[Type] {
        [and_token], [lifetime?], [mutability?], [elem],
    }

    struct TypeSlice[Type] {
        [bracket_token([elem])]
    }

    struct TypeTraitObject[Type] {
        [dyn_token?], [bounds*]
    }

    struct TypeTuple[Type] {
        [paren_token([elems*])]
    }

    struct ExprArray[Exempt] {
        [attrs*],
        [bracket_token([elems*])],
    }

    struct ExprAssign[Exempt] {
        [attrs*],
        [left],
        [eq_token],
        [right],
    }

    struct ExprAssignOp[Exempt] {
        [attrs*],
        [left],
        [op],
        [right],
    }

    struct ExprAsync[Exempt] {
        [attrs*], [async_token],
        [capture?], [block],
    }

    struct ExprAwait[Exempt] {
        [attrs*], [base], [dot_token],
        [await_token],
    }

    struct ExprBinary[Exempt] {
        [attrs*], [left], [op], [right],
    }

    struct ExprBlock[Exempt] {
        [attrs*], [label?], [block],
    }

    struct ExprBox[Exempt] {
        [attrs*], [box_token], [expr],
    }

    struct ExprBreak[Exempt] {
        [attrs*], [break_token], [label?],
        [expr?],
    }

    struct ExprCall[Exempt] {
        [attrs*], [func], [paren_token([args*])],
    }

    struct ExprCast[Exempt] {
        [attrs*], [expr], [as_token], [ty],
    }

    struct ExprClosure[Exempt] {
        [attrs*], [asyncness?], [movability?],
        [capture?], [or1_token], [inputs*],
        [or2_token], [output?], [body],
    }

    struct ExprContinue[Exempt] {
        [attrs*], [continue_token], [label?],
    }

    struct ExprField[Exempt] {
        [attrs*], [base], [dot_token], [member],
    }

    struct ExprForLoop[Exempt] {
        [attrs*], [label?], [for_token], [pat],
        [in_token], [expr], [body],
    }

    struct ExprGroup[Exempt] {
        [attrs*], [group_token([expr])]
    }

    struct ExprIndex[Exempt] {
        [attrs*], [expr],
        [bracket_token([index])],
    }

    struct ExprLet[Exempt] {
        [attrs*], [let_token], [pat], [eq_token],
        [expr],
    }

    struct ExprLit[Exempt] {
        [attrs*], [lit],
    }

    struct ExprLoop[Exempt] {
        [attrs*], [label?], [loop_token], [body],
    }

    struct ExprMacro[Exempt] {
        [attrs*], [mac],
    }

    struct ExprMatch[Exempt] {
        [attrs*], [match_token], [expr], [brace_token([arms*])]
    }

    struct ExprMethodCall[Exempt] {
        [attrs*], [receiver], [dot_token], [method],
        [turbofish?], [paren_token([args*])],
    }

    struct ExprParen[Exempt] {
        [attrs*], [paren_token([expr])],
    }

    struct ExprRange[Exempt] {
        [attrs*], [from?], [limits], [to?]
    }

    struct ExprReference[Exempt] {
        [attrs*], [and_token], [mutability?],
        [expr],
    }

    struct ExprRepeat[Exempt] {
        [attrs*],
        [bracket_token([expr], [semi_token], [len])],
    }

    struct ExprReturn[Exempt] {
        [attrs*], [return_token], [expr?],
    }

    struct ExprStruct[Exempt] {
        [attrs*], [path],
        [brace_token(
            [fields*], [dot2_token?],
            [rest?],
        )],
    }

    struct ExprTry[Exempt] {
        [attrs*], [expr], [question_token]
    }

    struct ExprTryBlock[Exempt] {
        [attrs*], [try_token], [block]
    }

    struct ExprTuple[Exempt] {
        [attrs*], [paren_token([elems*])]
    }

    struct ExprType[Exempt] {
        [attrs*], [expr], [colon_token],
        [ty],
    }

    struct ExprUnary[Exempt] {
        [attrs*], [op], [expr]
    }

    struct ExprUnsafe[Exempt] {
        [attrs*], [unsafe_token], [block],
    }

    struct ExprWhile[Exempt] {
        [attrs*], [label?], [while_token],
        [cond], [body],
    }

    struct ExprYield[Exempt] {
        [attrs*], [yield_token], [expr?],
    }

    struct PatBox[Exempt] {
        [attrs*], [box_token], [pat],
    }

    struct PatLit[Exempt] {
        [attrs*], [expr],
    }

    struct PatMacro[Exempt] {
        [attrs*], [mac],
    }

    struct PatOr[Exempt] {
        [attrs*], [leading_vert?],
        [cases*]
    }

    struct PatRange[Exempt] {
        [attrs*], [lo], [limits],
        [hi],
    }

    struct PatReference[Exempt] {
        [attrs*], [and_token],
        [mutability?], [pat],
    }

    struct PatRest[Exempt] {
        [attrs*], [dot2_token],
    }

    struct PatSlice[Exempt] {
        [attrs*], [bracket_token([elems*])]
    }

    struct PatStruct[Exempt] {
        [attrs*], [path],
        [brace_token([fields*], [dot2_token?])],
    }

    struct PatTuple[Exempt] {
        [attrs*], [paren_token([elems*])],
    }

    struct PatTupleStruct[Exempt] {
        [attrs*], [path], [pat],
    }

    struct PatType[Exempt] {
        [attrs*], [pat], [colon_token],
        [ty],
    }

    struct PatWild[Exempt] {
        [attrs*], [underscore_token],
    }

    struct FieldPat[Exempt] {
        [attrs*], [member], [colon_token?],
        [pat],
    }

    struct VisPublic[Exempt] { [pub_token] }

    struct VisCrate[Exempt] { [crate_token] }

    struct VisRestricted[Exempt] {
        [pub_token], [paren_token(
            [in_token?], [path]
        )]
    }

    struct Block[Exempt] {
        [brace_token([stmts*])]
    }

    struct Constraint[Exempt] {
        [ident], [colon_token], [bounds*]
    }

    struct PredicateType[Exempt] {
        [lifetimes?], [bounded_ty], [colon_token],
        [bounds*]
    }

    struct PredicateLifetime[Exempt] {
        [lifetime], [colon_token],
        [bounds*]
    }

    struct PredicateEq[Exempt] {
        [lhs_ty], [eq_token], [rhs_ty],
    }

    struct WhereClause[Exempt] {
        [where_token], [predicates*]
    }

    struct TypeParam[Exempt] {
        [attrs*], [ident], [colon_token?],
        [bounds*], [eq_token?], [default?],
    }

    struct ConstParam[Exempt] {
        [attrs*], [const_token], [ident],
        [colon_token], [ty], [eq_token?],
        [default?],
    }

    struct AngleBracketedGenericArguments[Exempt] {
        [colon2_token?], [lt_token], [args*],
        [gt_token],
    }

    struct ParenthesizedGenericArguments[Exempt] {
        [paren_token([inputs*])], [output?]
    }
}

simple_enum_lower_impl! {
    enum NestedMeta[Exempt] {
        Meta, Lit,
    }

    enum Meta[Meta] {
        Path, List, NameValue,
    }

    enum Member[Exempt] {
        Named, Unnamed,
    }

    enum RangeLimits[Exempt] {
        HalfOpen, Closed,
    }

    enum Expr[Expr Verbatim][__TestExhaustive] {
        Array, Assign, AssignOp, Async, Await,
        Binary, Block, Box, Break, Call, Cast,
        Closure, Continue, Field, ForLoop, Group,
        If, Index, Let, Lit, Loop, Macro, Match,
        MethodCall, Paren, Path, Range, Reference,
        Repeat, Return, Struct, Try, TryBlock,
        Tuple, Type, Unary, Unsafe, While, Yield,
    }

    enum BinOp[Exempt] {
        Add, Sub, Mul, Div, Rem, And, Or, BitXor,
        BitAnd, BitOr, Shl, Shr, Eq, Lt, Le, Ne,
        Ge, Gt, AddEq, SubEq, MulEq, DivEq, RemEq,
        BitXorEq, BitAndEq, BitOrEq, ShlEq, ShrEq,
    }

    enum UnOp[Exempt] {
        Deref, Not, Neg,
    }

    enum Pat[Pat Verbatim][__TestExhaustive] {
        Box, Ident, Lit, Macro, Or, Path, Range,
        Reference, Rest, Slice, Struct, Tuple,
        TupleStruct, Type, Wild,
    }

    enum Item[Item Verbatim][__TestExhaustive] {
        Const, Enum, ExternCrate, Fn, ForeignMod,
        Impl, Macro, Macro2, Mod, Static, Struct,
        Trait, TraitAlias, Type, Union, Use,
    }

    enum ForeignItem[Exempt Verbatim][__TestExhaustive] {
        Fn, Static, Type, Macro
    }

    enum ImplItem[Exempt Verbatim][__TestExhaustive] { // TODO maybe not exempt
        Const, Method, Type, Macro
    }

    enum TraitItem[Exempt Verbatim][__TestExhaustive] {
        Const, Method, Type, Macro
    }

    enum Type[Type Verbatim][__TestExhaustive] {
        Array, BareFn, Group, ImplTrait, Infer, Macro,
        Never, Paren, Path, Ptr, Reference, Slice,
        TraitObject, Tuple
    }

    enum GenericMethodArgument[GenericMethodArg] {
        Type, Const,
    }

    enum GenericArgument[GenericArg] {
        Lifetime, Type, Binding, Constraint,
        Const,
    }

    enum TypeParamBound[Exempt] {
        Trait, Lifetime
    }

    enum GenericParam[Exempt] {
        Type, Lifetime, Const
    }

    enum WherePredicate[Exempt] {
        Type, Lifetime, Eq
    }

    enum FnArg[Exempt] { Receiver, Typed }

    enum UseTree[Use] { Path, Name, Rename, Glob, Group }
}

impl Lower for Attribute {
    const RULE: R = R::Attribute;

    fn lower(self) -> Node {
        let mut children = vec![self.pound_token.lower()];
        self.style.lower_into(&mut children);
        let node = self.bracket_token.lower_start();

        {
            let mut children = node.children.borrow_mut();

            if let Ok(meta) = self.parse_meta() {
                children.push(meta.lower());
            } else {
                children.push(self.path.lower());
                children.push(self.tokens.lower());
            }
        }

        self.bracket_token.lower_end(&node);
        children.push(node);

        Node::simple(children)
    }
}

impl LowerOpt for PathArguments {
    fn lower_into(self, list: &mut Vec<Node>) {
        match self {
            Self::Parenthesized(v) => list.push(v.lower()),
            Self::AngleBracketed(v) => list.push(v.lower()),
            Self::None => {}
        }
    }
}

impl LowerOpt for ReturnType {
    #[inline]
    fn lower_into(self, list: &mut Vec<Node>) {
        if let Self::Type(arr, ty) = self {
            let mut node = Node::simple(vec![arr.lower(), ty.lower()]);
            node.optional = OptionalStatus::Optional;
            list.push(node);
        } else {
            return;
        };
    }
}

impl Lower for Stmt {
    const RULE: R = R::Stmt;
    fn lower(self) -> Node {
        let mut node = match self {
            Stmt::Expr(e) => e.lower(),
            Stmt::Item(i) => i.lower(),
            Stmt::Local(l) => l.lower(),
            Stmt::Semi(exp, semi) => Node::new(
                NodeKind::regular(),
                Self::RULE,
                vec![exp.lower(), semi.lower()],
            ),
        };

        node.rule = Self::RULE;

        node
    }
}

impl Lower for Local {
    const RULE: R = R::Local;
    fn lower(self) -> Node {
        let mut children = vec![
            self.attrs.lower_star(),
            self.let_token.lower(),
            self.pat.lower(),
        ];
        if let Some((eq, exp)) = self.init {
            children.push(eq.lower());
            children.push(exp.lower());
        }
        children.push(self.semi_token.lower());
        Node::new(NodeKind::regular(), Self::RULE, children)
    }
}

impl Lower for Arm {
    const RULE: R = R::Arm;
    fn lower(self) -> Node {
        let mut children = Vec::with_capacity(5);
        children.push(self.attrs.lower_star());
        children.push(self.pat.lower());
        if let Some((iif, exp)) = self.guard {
            children.push(iif.lower());
            children.push(exp.lower());
        }
        children.push(self.fat_arrow_token.lower());
        children.push(self.body.lower());
        self.comma.lower_into(&mut children);
        Node::new(NodeKind::regular(), Self::RULE, children)
    }
}

impl Lower for TraitBound {
    const RULE: R = R::Exempt;
    fn lower(self) -> Node {
        let node = if let Some(p) = &self.paren_token {
            p.lower_start()
        } else {
            Node::new(NodeKind::regular(), R::Exempt, vec![])
        };
        {
            let children = &mut node.children.borrow_mut();

            self.modifier.lower_into(children);
            self.lifetimes.lower_into(children);
            children.push(self.path.lower());
        }
        if let Some(p) = &self.paren_token {
            p.lower_end(&node);
        }
        node
    }
}

impl Lower for ExprIf {
    const RULE: R = R::Exempt;
    fn lower(self) -> Node {
        let mut children = vec![
            self.attrs.lower_star(),
            self.if_token.lower(),
            self.cond.lower(),
            self.then_branch.lower(),
        ];
        if let Some((elsee, expr)) = self.else_branch {
            children.push(elsee.lower());
            children.push(expr.lower());
        }
        Node::new(NodeKind::regular(), R::Exempt, children)
    }
}

impl Lower for PatIdent {
    const RULE: R = R::Exempt;
    fn lower(self) -> Node {
        let mut children = vec![self.attrs.lower_star()];
        self.by_ref.lower_into(&mut children);
        self.mutability.lower_into(&mut children);
        children.push(self.ident.lower());
        if let Some((at, pat)) = self.subpat {
            children.push(at.lower());
            children.push(pat.lower());
        }
        Node::new(NodeKind::regular(), R::Exempt, children)
    }
}

impl Lower for ExprPath {
    const RULE: R = R::Exempt;
    fn lower(self) -> Node {
        let children = vec![self.attrs.lower_star(), (self.qself, self.path).lower()];
        Node::new(NodeKind::regular(), R::Exempt, children)
    }
}

impl Lower for PatPath {
    const RULE: R = R::Exempt;
    fn lower(self) -> Node {
        let children = vec![self.attrs.lower_star(), (self.qself, self.path).lower()];
        Node::new(NodeKind::regular(), R::Exempt, children)
    }
}

impl Lower for TypePath {
    const RULE: R = R::Exempt;
    fn lower(self) -> Node {
        let children = vec![(self.qself, self.path).lower()];
        Node::new(NodeKind::regular(), R::Exempt, children)
    }
}

impl Lower for (Option<QSelf>, Path) {
    const RULE: R = R::QPath;
    fn lower(self) -> Node {
        // qself specifies the position of the cut between `P1` and `P2` in
        // `<Self as P1>::P2`, we need to lower it to kleene stars.

        let (qself, path) = self;
        let qself = if let Some(v) = qself {
            v
        } else {
            return path.lower();
        };

        let node = Node::new(NodeKind::regular(), Self::RULE, vec![]);
        {
            let mut children = node.children.borrow_mut();
            children.push(qself.lt_token.lower());
            children.push(qself.ty.lower());

            let pos = cmp::min(qself.position, path.segments.len());
            let mut segments = path.segments.into_pairs();
            if pos > 0 {
                children.push(qself.as_token.unwrap_or_default().lower());
                path.leading_colon.lower_into(&mut children);

                let segments_node = {
                    let mut separate = None;
                    let mut kleene = Vec::with_capacity(segments.len());
                    for (i, pair) in segments.by_ref().take(pos).enumerate() {
                        if i + 1 == pos {
                            separate = Some(pair.into_tuple());
                            break;
                        }
                        match pair {
                            Pair::Punctuated(v, p) => {
                                let vnode = v.lower();
                                let pnode = p.lower();
                                let tuple =
                                    Node::new(NodeKind::regular(), R::Exempt, vec![vnode, pnode]);
                                kleene.push(tuple);
                            }
                            Pair::End(_) => unreachable!(),
                        }
                    }
                    let mut children = vec![Node::new(NodeKind::KleeneStar, R::Exempt, kleene)];
                    let (path, punct) = separate.unwrap();

                    children.push(path.lower());
                    children.push(qself.gt_token.lower());
                    punct.lower_into(&mut children);

                    Node::new(NodeKind::regular(), R::Exempt, children)
                };

                children.push(segments_node);
            } else {
                children.push(qself.gt_token.lower());
                path.leading_colon.lower_into(&mut children);
            }

            let remaining = {
                let mut opt_trailing_punct = None;
                let mut kleene = Vec::with_capacity(segments.len());
                for pair in segments {
                    match pair {
                        Pair::Punctuated(v, p) => {
                            let vnode = v.lower();
                            let pnode = p.lower();
                            let tuple =
                                Node::new(NodeKind::regular(), R::Exempt, vec![vnode, pnode]);
                            kleene.push(tuple);
                        }
                        Pair::End(p) => opt_trailing_punct = Some(p),
                    }
                }
                let mut children = vec![Node::new(NodeKind::KleeneStar, R::Exempt, kleene)];
                opt_trailing_punct.lower_into(&mut children);

                Node::new(NodeKind::regular(), R::Exempt, children)
            };

            children.push(remaining);
        }

        node
    }
}

macro_rules! tuple_lower_impl {
    (
        $(
            ($ty1:ty, $ty2:ty)
        ),*$(,)?
    ) => {$(
        impl Lower for ($ty1, $ty2) {
            const RULE: R = R::Exempt;
            fn lower(self) -> Node {
                Node::new(NodeKind::regular(), R::Exempt, vec![self.0.lower(), self.1.lower()])
            }
        }
    )*};
}

tuple_lower_impl! {
    (Token![=], Expr),
    (Token![=], Type),
    (Token![as], Ident),
    (Ident, Token![:]),
}

impl Lower for (Token![&], Option<Lifetime>) {
    const RULE: R = R::Exempt;
    fn lower(self) -> Node {
        let mut children = vec![self.0.lower()];
        self.1.lower_into(&mut children);
        Node::new(NodeKind::regular(), R::Exempt, children)
    }
}

impl Lower for (Option<Token![!]>, Path, Token![for]) {
    const RULE: R = R::Exempt;
    fn lower(self) -> Node {
        let mut children = Vec::with_capacity(2);
        self.0.lower_into(&mut children);
        children.push(self.1.lower());
        children.push(self.2.lower());
        Node::new(NodeKind::regular(), R::Exempt, children)
    }
}

impl Lower for (token::Brace, Vec<Item>) {
    const RULE: R = R::Exempt;
    fn lower(self) -> Node {
        let node = self.0.lower_start();
        node.children.borrow_mut().push(self.1.lower_star());
        self.0.lower_end(&node);
        node
    }
}
