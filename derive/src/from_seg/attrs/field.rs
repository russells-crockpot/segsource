use super::toplevel::AlsoNeeds;
use crate::util::get_attr_value;
use alloc::rc::Rc;
use pmhelp::{
    exts::{GetBaseTypes as _, OptionTypeExt as _, ParseBufferExt as _},
    from_parens,
    parse::{
        parse_stream::comma_separated as comma_separated_ps,
        token_stream::{comma_separated, parenthesized},
    },
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
#[cfg(feature = "syn-full")]
use syn::ExprType;
use syn::{
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    token::Paren,
    Expr, ExprCall, ExprField, ExprLit, ExprPath, Field, Ident, Lit, LitInt, Path, Token, Type,
    TypePath,
};

mod kw {
    //TODO allow person to specify a path
    syn::custom_keyword!(parser);
    syn::custom_keyword!(move_to);
    syn::custom_keyword!(move_by);
    syn::custom_keyword!(as_is);
    syn::custom_keyword!(unwrap);
    syn::custom_keyword!(error_if);
    syn::custom_keyword!(skip);
    syn::custom_keyword!(from_iter);
    syn::custom_keyword!(map_each);
    syn::custom_keyword!(from);
    syn::custom_keyword!(try_from);
    syn::custom_keyword!(size);
    syn::custom_keyword!(default);
    syn::custom_keyword!(no_wrap);
    syn::custom_keyword!(also_pass);
    syn::custom_keyword!(parse_each);
    syn::custom_keyword!(remaining);
    syn::custom_keyword!(subseg);
}

pub struct AlsoPassEntry {
    expr: Box<Expr>,
    ty: Box<Type>,
}

#[cfg(feature = "syn-full")]
impl Parse for AlsoPassEntry {
    fn parse(stream: ParseStream) -> Result<Self> {
        let expr_type = stream.parse::<ExprType>()?;
        Ok(Self {
            expr: expr_type.expr,
            ty: expr_type.ty,
        })
    }
}

#[cfg(not(feature = "syn-full"))]
impl Parse for AlsoPassEntry {
    fn parse(stream: ParseStream) -> Result<Self> {
        Ok(Self {
            expr: stream.parse()?,
            ty: {
                stream.parse::<Token![:]>()?;
                stream.parse()?
            },
        })
    }
}

#[derive(Default)]
pub struct AlsoPass {
    suffix: Option<TokenStream>,
    subseg: Option<Box<Expr>>,
    segment_type: Option<TokenStream>,
    additional_types: Option<Punctuated<AlsoPassEntry, Token![,]>>,
}

impl AlsoPass {
    pub fn get_default_args(&self) -> TokenStream {
        if let Some(subseg) = &self.subseg {
            let suffix = &self.suffix;
            quote! {&segment.next_n(#subseg as usize)#suffix}
        } else {
            quote! {segment}
        }
    }

    pub fn set_segment_type(&mut self, segment_type: TokenStream) {
        if self.segment_type.is_some() {
            panic!("Attempted to set segment_type twice!")
        }
        self.segment_type = Some(segment_type);
    }

    pub fn get_conv_type(&self) -> TokenStream {
        let segment_type = self
            .segment_type
            .clone()
            .expect("segment_type has not been set!");
        if let Some(also_pass) = self.additional_types.as_ref() {
            if also_pass.is_empty() {
                segment_type
            } else {
                let types: Vec<&Box<Type>> = also_pass.iter().map(|an| &an.ty).collect();
                quote! {(#(#types,)* #segment_type)}
            }
        } else {
            segment_type
        }
    }

    pub fn get_args(&self) -> TokenStream {
        let default = self.get_default_args();
        if let Some(also_pass) = self.additional_types.as_ref() {
            if also_pass.is_empty() {
                default
            } else {
                let exprs: Vec<&Box<Expr>> = also_pass.iter().map(|an| &an.expr).collect();
                quote! {(#(#exprs,)* #default)}
            }
        } else {
            default
        }
    }
}

impl Parse for AlsoPass {
    #[inline]
    fn parse(stream: ParseStream) -> Result<Self> {
        Ok(Self {
            suffix: None,
            subseg: None,
            segment_type: None,
            additional_types: Some(comma_separated_ps(stream)?),
        })
    }
}

#[derive(PartialEq, Clone, Copy)]
enum TryOption {
    AsIs,
    Default,
    Try,
    Unwrap,
}

impl Parse for TryOption {
    fn parse(stream: ParseStream) -> Result<Self> {
        stream.peek_and_consume(Token![try]);
        stream.parse::<Token![=]>()?;
        if stream.peek_and_consume(kw::as_is) {
            Ok(Self::AsIs)
        } else if stream.peek_and_consume(Token![try]) {
            Ok(Self::Try)
        } else if stream.peek_and_consume(kw::unwrap) {
            Ok(Self::Unwrap)
        } else {
            Err(stream.error("Invalid choice for try"))
        }
    }
}

impl Default for TryOption {
    fn default() -> Self {
        TryOption::Default
    }
}

struct ErrorIf {
    predicate: Expr,
    error: Expr,
}
impl Parse for ErrorIf {
    fn parse(stream: ParseStream) -> Result<Self> {
        let stream = from_parens!(stream);
        let predicate = stream.parse()?;
        stream.parse::<Token![,]>()?;
        let error = stream.parse()?;
        Ok(Self { predicate, error })
    }
}

impl ToTokens for ErrorIf {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let predicate = &self.predicate;
        let error = &self.error;
        tokens.extend(quote! {
            if #predicate {
                return Err(#error)
            }
        });
    }
}

enum FromOption {
    Default,
    Type(Box<Type>),
}

impl Parse for FromOption {
    #[inline]
    fn parse(stream: ParseStream) -> Result<Self> {
        Ok(Self::Type(stream.parse()?))
    }
}

enum MapEachOption {
    Default,
    Expr(Box<Expr>),
}

impl Parse for MapEachOption {
    #[inline]
    fn parse(stream: ParseStream) -> Result<Self> {
        Ok(Self::Expr(stream.parse()?))
    }
}

enum SizeOption {
    Expr(Box<Expr>),
    Remaining,
}

impl SizeOption {
    fn get_loop_header(&self) -> TokenStream {
        match self {
            Self::Expr(value) => quote! { for _ in (0..#value) },
            Self::Remaining => quote! { while !segment.is_empty() },
        }
    }

    fn get_parse_each_tokens(&self) -> TokenStream {
        match self {
            Self::Expr(value) => quote! { .take(#value as usize) },
            Self::Remaining => quote! { .take_while(|_| !segment.is_empty()) },
        }
    }

    fn get_from_iter_tokens(&self) -> TokenStream {
        match self {
            Self::Expr(value) => value.to_token_stream(),
            Self::Remaining => quote! {segment.get_remaining()},
        }
    }
}

impl Parse for SizeOption {
    fn parse(stream: ParseStream) -> Result<Self> {
        if stream.peek_and_consume(kw::remaining) {
            Ok(Self::Remaining)
        } else {
            Ok(Self::Expr(stream.parse()?))
        }
    }
}

enum FromSegEntry {
    Skip,
    FromIter,
    MapEach(MapEachOption),
    From(FromOption),
    TryFrom(FromOption),
    Size(SizeOption),
    DefaultValue(Box<Expr>),
    Parser(Box<Expr>),
    Try(TryOption),
    If(Box<Expr>),
    ErrorIf(Box<ErrorIf>),
    NoWrap,
    AlsoPass(Box<AlsoPass>),
    MoveTo(Box<Expr>),
    MoveBy(Box<Expr>),
    Mut,
    ParseEach,
    Subseg(Box<Expr>),
    While(Box<Expr>),
}

impl FromSegEntry {
    fn apply(self, from_seg: &mut FromSegField) {
        match self {
            Self::Try(value) => from_seg.try_ = value,
            Self::From(value) => from_seg.from = Some(value),
            Self::TryFrom(value) => from_seg.try_from = Some(value),
            Self::Size(value) => from_seg.size = Some(value),
            Self::DefaultValue(value) => from_seg.default_value = Some(value),
            Self::Parser(value) => from_seg.parser = Some(value),
            Self::If(value) => from_seg.if_ = Some(value),
            Self::ErrorIf(value) => from_seg.error_if = Some(value),
            Self::AlsoPass(value) => from_seg.also_pass = value,
            Self::MoveTo(value) => from_seg.move_to = Some(value),
            Self::MoveBy(value) => from_seg.move_by = Some(value),
            Self::Subseg(value) => from_seg.subseg = Some(value),
            Self::While(value) => from_seg.take_while = Some(value),
            Self::MapEach(value) => {
                from_seg.from_iter = true;
                from_seg.map_each = Some(value);
            }
            Self::Skip => from_seg.skip = true,
            Self::FromIter => from_seg.from_iter = true,
            Self::NoWrap => from_seg.no_wrap = true,
            Self::Mut => from_seg.make_mut = true,
            Self::ParseEach => from_seg.parse_each = true,
        }
    }
}

impl Parse for FromSegEntry {
    fn parse(stream: ParseStream) -> Result<Self> {
        if stream.peek_and_consume(Token![try]) {
            Ok(Self::Try(stream.parse()?))
        } else if stream.peek_and_consume(Token![mut]) {
            Ok(Self::Mut)
        } else if stream.peek_and_consume(kw::no_wrap) {
            Ok(Self::NoWrap)
        } else if stream.peek_and_consume(kw::parse_each) {
            Ok(Self::ParseEach)
        } else if stream.peek_and_consume(Token![while]) {
            Ok(Self::While(from_parens!(stream).parse()?))
        // } else if stream.peek_and_consume(kw::subseg) {
        //     Ok(Self::Subseg(from_parens!(stream).parse()?))
        } else if stream.peek_and_consume(kw::parser) {
            Ok(Self::Parser(from_parens!(stream).parse()?))
        } else if stream.peek_and_consume(kw::also_pass) {
            Ok(Self::AlsoPass(from_parens!(stream).parse()?))
        } else if stream.peek_and_consume(kw::error_if) {
            Ok(Self::ErrorIf(stream.parse()?))
        } else if stream.peek_and_consume(Token![if]) {
            Ok(Self::If(from_parens!(stream).parse()?))
        } else if stream.peek_and_consume(kw::skip) {
            Ok(Self::Skip)
        } else if stream.peek_and_consume(kw::from_iter) {
            Ok(Self::FromIter)
        } else if stream.peek_and_consume(kw::move_to) {
            Ok(Self::MoveTo(from_parens!(stream).parse()?))
        } else if stream.peek_and_consume(kw::move_by) {
            Ok(Self::MoveBy(from_parens!(stream).parse()?))
        } else if stream.peek_and_consume(kw::size) {
            Ok(Self::Size(from_parens!(stream).parse()?))
        } else if stream.peek_and_consume(kw::default) {
            Ok(Self::DefaultValue(from_parens!(stream).parse()?))
        } else if stream.peek_and_consume(kw::map_each) {
            if stream.peek(Paren) {
                Ok(Self::MapEach(from_parens!(stream).parse()?))
            } else {
                Ok(Self::MapEach(MapEachOption::Default))
            }
        } else if stream.peek_and_consume(kw::from) {
            if stream.peek(Paren) {
                Ok(Self::From(from_parens!(stream).parse()?))
            } else {
                Ok(Self::From(FromOption::Default))
            }
        } else if stream.peek_and_consume(kw::try_from) {
            if stream.peek(Paren) {
                Ok(Self::TryFrom(from_parens!(stream).parse()?))
            } else {
                Ok(Self::TryFrom(FromOption::Default))
            }
        } else {
            Err(stream.error(format!("Invalid from_seg name: {}", stream)))
        }
    }
}

pub struct FromSegField {
    tmp_var: Ident,
    ty: Type,
    base_type: Option<Type>,
    generating_try_from: bool,
    also_needs: Rc<AlsoNeeds>,
    parser: Option<Box<Expr>>,
    try_from: Option<FromOption>,
    from: Option<FromOption>,
    size: Option<SizeOption>,
    try_: TryOption,
    skip: bool,
    map_each: Option<MapEachOption>,
    from_iter: bool,
    if_: Option<Box<Expr>>,
    error_if: Option<Box<ErrorIf>>,
    default_value: Option<Box<Expr>>,
    no_wrap: bool,
    also_pass: Box<AlsoPass>,
    move_to: Option<Box<Expr>>,
    move_by: Option<Box<Expr>>,
    subseg: Option<Box<Expr>>,
    take_while: Option<Box<Expr>>,
    make_mut: bool,
    parse_each: bool,
}

impl FromSegField {
    fn with_defaults(
        tmp_var: Ident,
        ty: Type,
        generating_try_from: bool,
        also_needs: Rc<AlsoNeeds>,
    ) -> Self {
        let base_type = if ty.is_option() {
            let base_types = ty.get_base_types();
            if base_types.is_empty() {
                panic!("Found an Option type, but couldn't determine its base type!");
            } else if base_types.len() > 1 {
                // Technically reachable if you made your own custom type named Option, but that's
                // a bad idea in general, so I'm not going to support it.
                unreachable!()
            } else {
                base_types.get(0).map(|ty| (*ty).clone())
            }
        } else {
            None
        };
        Self {
            tmp_var,
            ty,
            base_type,
            generating_try_from,
            also_needs,
            parser: None,
            try_from: None,
            from: None,
            size: None,
            try_: TryOption::Default,
            skip: false,
            map_each: None,
            from_iter: false,
            if_: None,
            error_if: None,
            default_value: None,
            no_wrap: false,
            also_pass: Default::default(),
            move_to: None,
            move_by: None,
            subseg: None,
            take_while: None,
            make_mut: false,
            parse_each: false,
        }
    }

    fn get_default_value(&self) -> TokenStream {
        self.default_value
            .as_ref()
            .map(ToTokens::to_token_stream)
            .unwrap_or(quote! {Default::default()})
    }

    fn get_assign_value(&self) -> TokenStream {
        let mut base = if self.skip {
            self.get_default_value()
        } else if self.parse_each {
            self.get_parse_each()
        } else if self.from_iter || self.map_each.is_some() {
            let map_each = self.get_map_each();
            let size = self.size.as_ref().unwrap().get_from_iter_tokens();
            let suffix = self.get_try_suffix_ignore_none();
            quote! {
                segment.next_n_as_slice(#size as usize)#suffix
                    .into_iter()
                    .cloned()
                    #map_each
                    .collect()
            }
        } else {
            self.get_simple_assign_val(self.also_pass.get_args())
        };
        if !self.no_wrap && self.ty.is_option() {
            base = quote! {Some(#base)};
        }
        if let Some(predicate) = &self.if_ {
            let default = self.get_default_value();
            quote! {
                if #predicate {
                    #base
                } else {
                    #default
                }
            }
        } else {
            base
        }
    }

    fn get_parse_each(&self) -> TokenStream {
        let take_while = self
            .take_while
            .as_ref()
            .map(|tw| quote! {.take_while(|value| #tw)});
        let map_each = self.get_map_each();
        let iter_def = if let Some(size) = &self.size {
            size.get_parse_each_tokens()
        } else if self.take_while.is_some() {
            quote! {.into_iter()}
        } else {
            unreachable!();
        };
        let gen_val = self.get_simple_assign_val_no_suffix(self.also_pass.get_args());
        if self.generating_try_from && matches!(self.try_, TryOption::Default | TryOption::Try) {
            let suffix = self.get_try_suffix_ignore_none();
            quote! {
                ::segsource::derive_extras::iter_to_result(::core::iter::repeat(true)
                    #iter_def
                    .map(|_| #gen_val)
                    #take_while
                )#suffix
                #map_each
                .collect()
            }
        } else {
            quote! {
                ::core::iter::repeat(true)
                #iter_def
                .map(|_| -> #gen_val)
                #take_while
                #map_each
                .collect()
            }
        }
    }

    fn get_simple_assign_val(&self, value: TokenStream) -> TokenStream {
        let assign_val = self.get_simple_assign_val_no_suffix(value);
        let suffix = self.get_try_suffix();
        quote! {#assign_val#suffix}
    }

    /// Gets the value to be assigned, ignoring parse_each values
    fn get_simple_assign_val_no_suffix(&self, value: TokenStream) -> TokenStream {
        let conv_type = &self.also_pass.get_conv_type();
        if let Some(parser) = &self.parser {
            quote! {#parser}
        } else if let Some(FromOption::Default) = &self.from {
            let ty = self.base_type.as_ref().unwrap_or(&self.ty);
            quote! {<#ty as ::core::convert::From<#conv_type>>::from(#value)}
        } else if let Some(FromOption::Type(ty)) = &self.from {
            quote! {<#ty as ::core::convert::From<#conv_type>>::from(#value)}
        } else if let Some(FromOption::Default) = &self.try_from {
            let ty = self.base_type.as_ref().unwrap_or(&self.ty);
            quote! {<#ty as ::core::convert::TryFrom<#conv_type>>::try_from(#value)}
        } else if let Some(FromOption::Type(ty)) = &self.try_from {
            quote! {<#ty as ::core::convert::TryFrom<#conv_type>>::try_from(#value)}
        } else if self.generating_try_from {
            let ty = self.base_type.as_ref().unwrap_or(&self.ty);
            quote! {<#ty as ::core::convert::TryFrom<#conv_type>>::try_from(#value)}
        } else {
            let ty = self.base_type.as_ref().unwrap_or(&self.ty);
            quote! {<#ty as ::core::convert::From<#conv_type>>::from(#value)}
        }
    }

    fn get_map_each(&self) -> Option<TokenStream> {
        match &self.map_each {
            None => None,
            Some(MapEachOption::Default) => {
                let value = self.get_simple_assign_val(quote! {item});
                //TODO add clippy allows
                Some(quote! {.map(|item| #value)})
            }
            Some(MapEachOption::Expr(value)) => Some(quote!(.map(#value))),
        }
    }

    fn get_try_suffix_ignore_none(&self) -> TokenStream {
        match self.try_ {
            TryOption::Try => quote! {?},
            TryOption::Default | TryOption::AsIs if self.generating_try_from => quote! {?},
            TryOption::Unwrap => quote! {.unwrap()},
            TryOption::Default | TryOption::AsIs if !self.generating_try_from => {
                quote! {.unwrap()}
            }
            _ => unreachable!(),
        }
    }

    fn get_try_suffix(&self) -> Option<TokenStream> {
        match self.try_ {
            TryOption::AsIs => None,
            _ => Some(self.get_try_suffix_ignore_none()),
        }
    }

    fn get_post_statements(&self) -> TokenStream {
        let error_if = &self.error_if;
        quote! {
            #error_if
        }
    }

    fn get_pre_assign_statements(&self) -> Option<TokenStream> {
        let suffix = if self.generating_try_from {
            quote! {?}
        } else {
            quote! {.unwrap()}
        };
        if self.move_to.is_some() {
            self.move_to
                .as_ref()
                .map(|expr| (quote! {segment.move_to(#expr)#suffix;}))
        } else if self.move_by.is_some() {
            self.move_by
                .as_ref()
                .map(|expr| (quote! {segment.move_by(#expr)#suffix;}))
        } else {
            None
        }
    }

    pub fn tmp_var(&self) -> Ident {
        self.tmp_var.clone()
    }
}

impl From<(usize, (Field, bool, Rc<AlsoNeeds>))> for FromSegField {
    fn from(
        (idx, (field, generating_try_from, also_needs)): (usize, (Field, bool, Rc<AlsoNeeds>)),
    ) -> Self {
        let tmp_var = field
            .ident
            .clone()
            .unwrap_or(quote::format_ident!("tmp_{}", idx));
        let mut me =
            FromSegField::with_defaults(tmp_var, field.ty, generating_try_from, also_needs);
        for attr in field.attrs {
            if attr.path.is_ident("from_seg") {
                comma_separated::<FromSegEntry>(parenthesized::<TokenStream>(attr.tokens).unwrap())
                    .unwrap()
                    .into_iter()
                    .for_each(|entry| entry.apply(&mut me));
                break;
            }
        }
        if me.default_value.is_some() && (me.if_.is_none() || !me.skip) {
            panic!("Cannot specify default_value without an if condition or skip!")
        }
        if (me.map_each.is_some() || me.from_iter || me.parse_each)
            && me.size.is_none()
            && me.take_while.is_none()
        {
            panic!("A size or take_while is needed for repeated values!")
        }
        if me.move_to.is_some() && me.move_by.is_some() {
            panic!("Either move_to or move_by can be specified, not both.")
        }
        me.also_pass.segment_type = Some(me.also_needs.get_segment_type());
        me.also_pass.suffix = Some(me.get_try_suffix_ignore_none());
        if me.parse_each {
            let bt = if let Some(bt) = me.base_type {
                bt
            } else {
                me.ty.clone()
            };
            let base_types = bt.get_base_types();
            if base_types.is_empty() {
                panic!("Couldn't determine the base type of {}!", me.tmp_var);
            } else if base_types.len() > 1 {
                //TODO consider assuming it's a tuple of the type?
                panic!("More than one base type found for field {}!", me.tmp_var);
            } else {
                me.base_type = base_types.get(0).map(|ty| (*ty).clone())
            }
        }
        if me.subseg.is_some() {
            me.also_pass.subseg = me.subseg;
            me.subseg = None;
        }
        me
    }
}

impl ToTokens for FromSegField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let tmp_var = &self.tmp_var;
        let pre_assign = self.get_pre_assign_statements();
        let lhs = self.get_assign_value();
        let post_statements = self.get_post_statements();
        let make_mut = if self.make_mut {
            Some(quote! {mut})
        } else {
            None
        };
        let result = quote! {
            #pre_assign
            let #make_mut #tmp_var = #lhs;
            #post_statements
        };
        tokens.extend(result);
    }
}
