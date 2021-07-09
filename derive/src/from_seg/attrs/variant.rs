#![allow(dead_code, unused_variables, unused_imports, unused_macros)]
use crate::util::{get_attr_value, parse_parenthesized2};
use core::iter::{FromIterator, IntoIterator};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream, Result as ParseResult},
    punctuated::Punctuated,
    Expr, ExprCall, ExprField, ExprLit, ExprPath, Field, Ident, Lit, LitInt, Path, Token, Type,
};

mod kw {
    syn::custom_keyword!(peek);
    syn::custom_keyword!(has_more);
    syn::custom_keyword!(value_is);
    syn::custom_keyword!(predicate);
    syn::custom_keyword!(default);
}

enum AtOption {
    Next,
    Offset(LitInt),
    Expr(Box<Expr>),
}

impl Default for AtOption {
    fn default() -> Self {
        Self::Next
    }
}
impl Parse for AtOption {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        match stream.parse::<Expr>()? {
            Expr::Lit(ExprLit {
                lit: Lit::Int(val), ..
            }) => Ok(Self::Offset(val)),
            Expr::Path(ExprPath { path, .. }) if path.is_ident("next") => Ok(Self::Next),
            other => Ok(Self::Expr(Box::new(other))),
        }
    }
}

struct ValueIs {
    type_: Type,
    value: Expr,
    consume: bool,
    at: AtOption,
}

impl Parse for ValueIs {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        let type_ = stream.parse::<Type>()?;
        stream.parse::<Token![,]>()?;
        let value = stream.parse::<Expr>()?;
        let mut peek = false;
        let mut at = AtOption::default();
        if stream.peek(Token![,]) {
            stream.parse::<Token![,]>()?;
            if stream.peek(kw::peek) {
                stream.parse::<kw::peek>()?;
                peek = true;
            } else {
                at = stream.parse::<AtOption>()?;
            }
            if stream.peek(Token![,]) {
                stream.parse::<Token![,]>()?;
                if stream.peek(kw::peek) {
                    stream.parse::<kw::peek>()?;
                    peek = true;
                } else {
                    at = stream.parse::<AtOption>()?;
                }
            }
        }
        Ok(Self {
            type_,
            value,
            consume: !peek,
            at,
        })
    }
}

enum UseIfEntry {
    HasMore,
    Default,
    ValueIs(Box<ValueIs>),
    Predicate(Path),
}

impl Parse for UseIfEntry {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        if stream.peek(kw::has_more) {
            Ok(Self::HasMore)
        } else if stream.peek(kw::default) {
            Ok(Self::Default)
        } else if stream.peek(kw::value_is) {
            stream.parse::<kw::value_is>()?;
            Ok(Self::ValueIs(Box::new(parse_parenthesized2(stream)?)))
        } else if stream.peek(kw::predicate) {
            stream.parse::<kw::predicate>()?;
            Ok(Self::Predicate(stream.parse()?))
        } else {
            Err(stream.error(format!("Invalid input to use_if: {}", stream)))
        }
    }
}
