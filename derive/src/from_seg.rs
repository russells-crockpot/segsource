use crate::util::{create_new_lifetimes, parse_parenthesized};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    punctuated::Punctuated, DeriveInput, Fields, FieldsNamed, FieldsUnnamed, Ident, Token, Type,
};
mod attr;
use attr::FromSegField;

pub(crate) fn base_from_segment(input: DeriveInput, generating_try_from: bool) -> TokenStream {
    let item_type_attr_name = if generating_try_from {
        "try_from_item_type"
    } else {
        "from_item_type"
    };
    let (tuple_like, fields_iter) = if let syn::Data::Struct(body) = input.data {
        match body.fields {
            Fields::Named(FieldsNamed { named, .. }) => (false, named.into_iter()),
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => (true, unnamed.into_iter()),
            Fields::Unit => panic!("Struct {} has no fields!", input.ident),
        }
    } else {
        panic!("Items can only be derived from a struct!");
    };
    let mut maybe_item_type = None;
    let mut maybe_error_stmt = None;
    for attr in input.attrs {
        if attr.path.is_ident(item_type_attr_name) {
            maybe_item_type = Some(parse_parenthesized::<Type>(attr.tokens).unwrap());
        } else if attr.path.is_ident("try_from_error") {
            let error_type = Some(parse_parenthesized::<Type>(attr.tokens).unwrap());
            maybe_error_stmt = Some(quote! { type Error = #error_type; });
        }
    }
    let item_type =
        maybe_item_type.unwrap_or_else(|| panic!("No {} attribute found!", item_type_attr_name));
    if generating_try_from && maybe_error_stmt.is_none() {
        panic!("No try_from_error attribute found!")
    }
    let name = input.ident;
    let (_, type_g, _) = input.generics.split_for_impl();
    let ([lifetime], generics) = create_new_lifetimes(&input.generics);
    let (impl_g, _, maybe_where) = generics.split_for_impl();
    let segment_type = quote! {&::segsource::Segment<#lifetime, #item_type>};
    let fields: Punctuated<FromSegField, Token![;]> = fields_iter
        .map(|f| (f, generating_try_from, segment_type.clone()))
        .enumerate()
        .map(FromSegField::from)
        .collect();
    let field_names: Punctuated<Ident, Token![,]> =
        fields.iter().map(FromSegField::tmp_var).collect();
    let (trait_name, method_sig) = if generating_try_from {
        (
            quote! {::std::convert::TryFrom},
            quote! {
                try_from(segment: #segment_type)
                    -> ::std::result::Result<Self, Self::Error>
            },
        )
    } else {
        (
            quote! {::std::convert::From},
            quote! {from(segment: #segment_type) -> Self},
        )
    };
    let create_self_stmt = if tuple_like && generating_try_from {
        quote! {Ok(Self(#field_names))}
    } else if tuple_like {
        quote! {Self(#field_names)}
    } else if generating_try_from {
        quote! {Ok(Self{#field_names})}
    } else {
        quote! {Self{#field_names}}
    };
    quote! {
        impl #impl_g #trait_name<#segment_type> for #name #type_g #maybe_where {
            #maybe_error_stmt
            fn #method_sig {
                #fields
                #create_self_stmt
            }
        }
    }
}

pub(crate) fn derive_from_segment(input: DeriveInput) -> TokenStream {
    base_from_segment(input, false)
}

pub(crate) fn derive_try_from_segment(input: DeriveInput) -> TokenStream {
    base_from_segment(input, true)
}
