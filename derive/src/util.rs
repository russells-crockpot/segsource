use proc_macro2::{Span, TokenStream};
use std::{convert::TryFrom, iter::Cycle, ops::RangeInclusive};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream, Result as ParseResult},
    GenericParam, Generics, Lifetime, LifetimeDef, Token,
};

pub struct LifetimeGenerator(Vec<(char, Cycle<RangeInclusive<char>>)>);

impl LifetimeGenerator {
    fn new() -> Self {
        Self(vec![Self::new_item()])
    }

    fn new_item() -> (char, Cycle<RangeInclusive<char>>) {
        let mut cycle = ('a'..='z').cycle();
        cycle.next();
        ('a', cycle)
    }

    fn get_current_string(&self) -> String {
        self.0.iter().map(|(c, _)| *c).collect()
    }

    fn inc_combo(&mut self, idx: usize) {
        let value = self.0.get_mut(idx).unwrap();
        let next = value.1.next().unwrap();
        value.0 = next;
        if next == 'a' {
            if idx == 0 {
                self.0.push(Self::new_item());
            } else {
                self.inc_combo(idx - 1);
            }
        }
    }
}

impl Iterator for LifetimeGenerator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let rval = self.get_current_string();
        self.inc_combo(self.0.len() - 1);
        Some(format!("'{}", rval))
    }
}

impl Default for LifetimeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

pub fn create_new_lifetimes<const N: usize>(base: &Generics) -> ([Lifetime; N], Generics) {
    let mut seen_lifetimes = Vec::new();
    let mut insert_at = 0;
    for (i, param) in base.params.iter().enumerate() {
        if let GenericParam::Lifetime(ref lifetime) = param {
            seen_lifetimes.push(format!("{}", lifetime.lifetime.ident));
        } else {
            insert_at = i;
            break;
        }
    }
    let mut new_lifetimes = Vec::with_capacity(N);
    let mut lifetime_gen = LifetimeGenerator::new();
    while new_lifetimes.len() < N {
        let lf_name = lifetime_gen.next().unwrap();
        if !seen_lifetimes.contains(&lf_name) {
            let lifetime = Lifetime::new(&lf_name, Span::call_site());
            new_lifetimes.push(lifetime);
        }
    }
    let mut generics = base.clone();
    for lf in new_lifetimes.iter() {
        let param = GenericParam::Lifetime(LifetimeDef::new(lf.clone()));
        generics.params.insert(insert_at, param);
    }
    let new_lifetimes = if let Ok(a) = <[Lifetime; N]>::try_from(new_lifetimes) {
        a
    } else {
        unreachable!();
    };
    (new_lifetimes, generics)
}

pub struct Parenthesized<V: Parse>(V);
impl<V: Parse> Parse for Parenthesized<V> {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        let content;
        let _ = parenthesized!(content in stream);
        Ok(Self(V::parse(&content)?))
    }
}

pub fn parse_parenthesized<V: Parse>(stream: TokenStream) -> syn::parse::Result<V> {
    syn::parse2::<Parenthesized<V>>(stream).map(|v| v.0)
}

pub fn parse_parenthesized2<V: Parse>(stream: ParseStream) -> syn::parse::Result<V> {
    stream.parse::<Parenthesized<V>>().map(|v| v.0)
}

pub fn get_attr_value<P: Parse>(stream: ParseStream) -> ParseResult<P> {
    stream.parse::<Token![=]>()?;
    stream.parse::<P>()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_lifetime_generator() {
        let mut gen = LifetimeGenerator::new();
        for c in 'a'..='z' {
            assert_eq!(gen.next().unwrap(), format!("'{}", c));
        }
        for c in 'a'..='z' {
            assert_eq!(gen.next().unwrap(), format!("'a{}", c));
        }
        for c in 'a'..='z' {
            assert_eq!(gen.next().unwrap(), format!("'b{}", c));
        }
    }
}
