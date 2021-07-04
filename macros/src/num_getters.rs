//use std::string::ToString;
use proc_macro2::{
    Group as PmGroup, Ident as PmIdent, Literal as PmLiteral, Span, TokenStream, TokenTree,
};
use quote::quote;
use syn::{Ident, Lit};

#[derive(Clone, Copy)]
pub enum Endidness {
    Big,
    Little,
    Native,
}

impl Endidness {
    fn short(&self) -> &str {
        match self {
            Self::Big => "be",
            Self::Little => "le",
            Self::Native => "ne",
        }
    }
    fn long(&self) -> &str {
        match self {
            Self::Big => "big",
            Self::Little => "little",
            Self::Native => "native",
        }
    }
    fn title(&self) -> &str {
        match self {
            Self::Big => "Big",
            Self::Little => "Little",
            Self::Native => "Native",
        }
    }
}

struct NumberInfo {
    ident: &'static str,
    width: u8,
}

impl NumberInfo {
    fn new(ident: &'static str, width: u8) -> Self {
        Self { ident, width }
    }

    fn width(&self) -> TokenTree {
        TokenTree::Literal(PmLiteral::u8_unsuffixed(self.width))
    }

    fn ident(&self) -> TokenTree {
        TokenTree::Ident(PmIdent::new(self.ident, Span::call_site()))
    }

    fn apply_to_stream(&self, stream: TokenStream, endidness: Endidness) -> TokenStream {
        stream
            .into_iter()
            .map(|token| self.apply_to_tree(token, endidness))
            .collect()
    }

    fn alter_str(&self, value: &str, endidness: Endidness) -> String {
        value
            .replace("numend", endidness.short())
            .replace("numname", self.ident)
            .replace("numwidth", &self.width.to_string())
            .replace("numendlong", endidness.long())
            .replace("numendtitle", endidness.title())
    }

    fn apply_to_tree(&self, tree: TokenTree, endidness: Endidness) -> TokenTree {
        match tree {
            TokenTree::Group(group) => TokenTree::Group(PmGroup::new(
                group.delimiter(),
                self.apply_to_stream(group.stream(), endidness),
            )),
            TokenTree::Ident(ident) => {
                let ident_str = ident.to_string();
                if ident_str == "_numname_" {
                    self.ident()
                } else if ident_str == "_numwidth_" {
                    self.width()
                } else if ident_str == "_numendlong_" {
                    TokenTree::Ident(PmIdent::new(endidness.long(), Span::call_site()))
                } else if ident_str == "_numendtitle_" {
                    TokenTree::Ident(PmIdent::new(endidness.title(), Span::call_site()))
                } else {
                    TokenTree::Ident(PmIdent::new(
                        &self.alter_str(&ident_str, endidness),
                        ident.span(),
                    ))
                }
            }
            TokenTree::Literal(lit) => match Lit::new(lit.clone()) {
                Lit::Str(s) => {
                    TokenTree::Literal(PmLiteral::string(&self.alter_str(&s.value(), endidness)))
                }
                _ => TokenTree::Literal(lit),
            },
            _ => tree,
        }
    }
}

lazy_static! {
    static ref I8_NUM_INFO: NumberInfo = NumberInfo::new("i8", 1);
    static ref NUMBERS: Vec<NumberInfo> = vec![
        NumberInfo::new("u16", 2),
        NumberInfo::new("i16", 2),
        NumberInfo::new("u32", 4),
        NumberInfo::new("i32", 4),
        NumberInfo::new("u64", 8),
        NumberInfo::new("i64", 8),
        NumberInfo::new("u128", 16),
        NumberInfo::new("i128", 16),
    ];
}

pub fn impl_next_methods(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let in_stream = TokenStream::from(stream);
    let mut out = generate_next_impl(&I8_NUM_INFO, Endidness::Native, &in_stream);
    for num_info in NUMBERS.iter() {
        out.extend(generate_next_impl(num_info, Endidness::Little, &in_stream));
        out.extend(generate_next_impl(num_info, Endidness::Big, &in_stream));
    }
    out.into()
}

fn generate_next_impl(
    num_info: &NumberInfo,
    endidness: Endidness,
    stream: &TokenStream,
) -> TokenStream {
    let impl_meth_name = Ident::new(
        &format!("next_{}_{}", num_info.ident, endidness.short()),
        Span::call_site(),
    );
    let body = num_info.apply_to_stream(stream.clone(), endidness);
    let rtype = num_info.ident();
    quote! {
        fn #impl_meth_name(&mut self) -> segsource::Result<#rtype> {
            #body
        }
    }
}

pub fn impl_at_methods(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let in_stream = TokenStream::from(stream);
    let mut out = generate_at_impl(&I8_NUM_INFO, Endidness::Native, &in_stream);
    for num_info in NUMBERS.iter() {
        out.extend(generate_at_impl(num_info, Endidness::Little, &in_stream));
        out.extend(generate_at_impl(num_info, Endidness::Big, &in_stream));
    }
    out.into()
}

fn generate_at_impl(
    num_info: &NumberInfo,
    endidness: Endidness,
    stream: &TokenStream,
) -> TokenStream {
    let impl_meth_name = Ident::new(
        &format!("{}_{}_at", num_info.ident, endidness.short()),
        Span::call_site(),
    );
    let body = num_info.apply_to_stream(stream.clone(), endidness);
    let rtype = num_info.ident();
    quote! {
        fn #impl_meth_name(&self, offset: usize) -> segsource::Result<#rtype> {
            #body
        }
    }
}

fn for_each_number_with_endidness(stream: &TokenStream, endidness: Endidness) -> TokenStream {
    let mut out = TokenStream::new();
    for num_info in NUMBERS.iter() {
        out.extend(num_info.apply_to_stream(stream.clone(), endidness));
    }
    out
}

pub fn make_number_methods(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let in_stream = TokenStream::from(stream);
    let mut out = I8_NUM_INFO.apply_to_stream(in_stream.clone(), Endidness::Native);
    out.extend(for_each_number_with_endidness(&in_stream, Endidness::Big));
    out.extend(for_each_number_with_endidness(
        &in_stream,
        Endidness::Little,
    ));
    out.into()
}

pub fn for_each_number(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let in_stream = TokenStream::from(stream);
    for_each_number_with_endidness(&in_stream, Endidness::Native).into()
}
