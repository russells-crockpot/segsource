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
    FieldName(Ident),
    Field(ExprField),
    Constant(LitInt),
    Call(ExprCall),
    Remaining,
}

impl Parse for SizeOption {
    fn parse(stream: ParseStream) -> Result<Self> {
        match Expr::parse(stream)? {
            Expr::Lit(ExprLit {
                lit: Lit::Int(int), ..
            }) => Ok(Self::Constant(int)),
            Expr::Path(ExprPath { path, .. }) => {
                if let Some(ident) = path.get_ident() {
                    if ident == "remaining" {
                        Ok(Self::Remaining)
                    } else {
                        Ok(Self::FieldName(ident.clone()))
                    }
                } else {
                    Err(stream.error(format!(
                        "Invalid value {} for size option.",
                        path.to_token_stream()
                    )))
                }
            }
            Expr::Field(field) => Ok(Self::Field(field)),
            Expr::Call(call) => Ok(Self::Call(call)),
            other => Err(stream.error(format!(
                "Invalid value {} for size option.",
                other.to_token_stream()
            ))),
        }
    }
}
impl ToTokens for SizeOption {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::FieldName(value) => value.to_tokens(tokens),
            Self::Field(value) => value.to_tokens(tokens),
            Self::Constant(value) => value.to_tokens(tokens),
            Self::Call(value) => value.to_tokens(tokens),
            Self::Remaining => todo!(),
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
    DefaultValue(Expr),
    Parser(Expr),
    Try(TryOption),
    If(Expr),
    ErrorIf(Box<ErrorIf>),
    NoWrap,
    AlsoPass(Punctuated<Expr, Token![,]>),
    MoveTo(Expr),
    MoveBy(Expr),
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
            Self::AlsoPass(value) => from_seg.also_pass = Some(value),
            Self::MoveTo(value) => from_seg.move_to = Some(value),
            Self::MoveBy(value) => from_seg.move_by = Some(value),
            Self::MapEach(value) => {
                from_seg.from_iter = true;
                from_seg.map_each = Some(value);
            }
            Self::Skip => from_seg.skip = true,
            Self::FromIter => from_seg.from_iter = true,
            Self::NoWrap => from_seg.no_wrap = true,
        }
    }
}

impl Parse for FromSegEntry {
    fn parse(stream: ParseStream) -> Result<Self> {
        if stream.peek_and_consume(Token![try]) {
            Ok(Self::Try(stream.parse()?))
        } else if stream.peek_and_consume(kw::no_wrap) {
            stream.parse::<kw::no_wrap>()?;
            Ok(Self::NoWrap)
        } else if stream.peek_and_consume(kw::parser) {
            Ok(Self::Parser(from_parens!(stream).parse()?))
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
    parser: Option<Expr>,
    try_from: Option<FromOption>,
    from: Option<FromOption>,
    size: Option<SizeOption>,
    try_: TryOption,
    skip: bool,
    map_each: Option<MapEachOption>,
    from_iter: bool,
    if_: Option<Expr>,
    error_if: Option<Box<ErrorIf>>,
    default_value: Option<Expr>,
    no_wrap: bool,
    also_pass: Option<Punctuated<Expr, Token![,]>>,
    move_to: Option<Expr>,
    move_by: Option<Expr>,
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
            also_pass: None,
            move_to: None,
            move_by: None,
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
        } else if self.from_iter || self.map_each.is_some() {
            let map_each = self.get_map_each();
            let size = &self.size;
            let suffix = self.get_try_suffix_ignore_none();
            quote! {
                segment.next_n_as_slice(#size as usize)#suffix
                    .into_iter()
                    .cloned()
                    #map_each
                    .collect()
            }
        } else {
            self.get_simple_assign_val(quote! {segment})
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

    /// Gets the value to be assigned, ignoring repeated values
    fn get_simple_assign_val(&self, value: TokenStream) -> TokenStream {
        let suffix = self.get_try_suffix();
        let segment_type = &self.also_needs.get_segment_type();
        if let Some(parser) = &self.parser {
            quote! {#parser#suffix}
        } else if let Some(FromOption::Default) = &self.from {
            let ty = self.base_type.as_ref().unwrap_or(&self.ty);
            quote! {<#ty as ::core::convert::From<#segment_type>>::from(#value)}
        } else if let Some(FromOption::Type(ty)) = &self.from {
            quote! {<#ty as ::core::convert::From<#segment_type>>::from(#value)}
        } else if let Some(FromOption::Default) = &self.try_from {
            let ty = self.base_type.as_ref().unwrap_or(&self.ty);
            quote! {<#ty as ::core::convert::TryFrom<#segment_type>>::try_from(#value)#suffix}
        } else if let Some(FromOption::Type(ty)) = &self.try_from {
            quote! {<#ty as ::core::convert::TryFrom<#segment_type>>::try_from(#value)#suffix}
        } else if self.generating_try_from {
            let ty = self.base_type.as_ref().unwrap_or(&self.ty);
            quote! {<#ty as ::core::convert::TryFrom<#segment_type>>::try_from(#value)#suffix}
        } else {
            let ty = self.base_type.as_ref().unwrap_or(&self.ty);
            quote! {<#ty as ::core::convert::From<#segment_type>>::from(#value)}
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
        if (me.map_each.is_some() || me.from_iter) && me.size.is_none() {
            panic!("A size is needed for repeated values!")
        }
        if me.move_to.is_some() && me.move_by.is_some() {
            panic!("move_to or move_by can be specified, not both.")
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
        let result = quote! {
            #pre_assign
            let #tmp_var = #lhs;
            #post_statements
        };
        tokens.extend(result);
    }
}
