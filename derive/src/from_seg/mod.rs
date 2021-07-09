use alloc::rc::Rc;
use pmhelp::{parse::token_stream::parenthesized, util::create_new_lifetimes};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parser as _, Result},
    punctuated::Punctuated,
    Data, DataEnum, DataStruct, DeriveInput, Fields, FieldsNamed, FieldsUnnamed, Ident, Token,
    Type,
};
mod attrs;
use attrs::{AlsoNeeds, FromSegField, FromSegInfo};

fn generate_fields_body(
    ident: &Ident,
    fields: Fields,
    also_needs: Rc<AlsoNeeds>,
    generating_try_from: bool,
) -> TokenStream {
    let (tuple_like, fields_iter) = match fields {
        Fields::Named(FieldsNamed { named, .. }) => (false, named.into_iter()),
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => (true, unnamed.into_iter()),
        Fields::Unit => panic!("Struct or variant {} has no fields!", ident),
    };
    let fields: Punctuated<FromSegField, Token![;]> = fields_iter
        .map(|f| (f, generating_try_from, Rc::clone(&also_needs)))
        .enumerate()
        .map(FromSegField::from)
        .collect();
    let field_names: Punctuated<Ident, Token![,]> =
        fields.iter().map(FromSegField::tmp_var).collect();
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
        #fields
        #create_self_stmt
    }
}

fn generate_body(
    ident: &Ident,
    data: Data,
    also_needs: Rc<AlsoNeeds>,
    generating_try_from: bool,
) -> TokenStream {
    match data {
        Data::Struct(DataStruct { fields, .. }) => {
            generate_fields_body(ident, fields, also_needs, generating_try_from)
        }
        Data::Enum(DataEnum { variants, .. }) => todo!(),
        _ => unimplemented!(),
    }
}

pub fn base_from_segment(input: DeriveInput, generating_try_from: bool) -> Result<TokenStream> {
    let mut maybe_info = None;
    for attr in input.attrs {
        if attr.path.is_ident("from_seg") {
            maybe_info = Some(syn::parse2(parenthesized::<TokenStream>(attr.tokens)?)?);
            break;
        }
    }
    let ([lifetime], generics) = create_new_lifetimes(&input.generics);
    let (impl_g, _, maybe_where) = generics.split_for_impl();
    let FromSegInfo {
        item_type,
        error_type,
        mut also_needs,
    } = maybe_info.unwrap_or_default();
    if generating_try_from && error_type.is_none() {
        panic!("No error type specified!");
    }
    also_needs.set_segment_generics(quote! {#lifetime, #item_type});
    let also_needs = Rc::new(also_needs);
    let error_stmt = error_type
        .map(|etype| quote! {type Error = #etype;})
        .unwrap_or_default();
    let name = input.ident;
    let (_, type_g, _) = input.generics.split_for_impl();
    let segment_type = also_needs.get_type();
    let method_args = also_needs.get_args();
    let (trait_name, method_sig) = if generating_try_from {
        (
            quote! {::core::convert::TryFrom},
            quote! {
                try_from(#method_args: #segment_type)
                    -> ::core::result::Result<Self, Self::Error>
            },
        )
    } else {
        (
            quote! {::core::convert::From},
            quote! {from(segment: #segment_type) -> Self},
        )
    };
    let body = generate_body(&name, input.data, also_needs, generating_try_from);
    Ok(quote! {
        impl #impl_g #trait_name<#segment_type> for #name #type_g #maybe_where {
            #error_stmt
            #[allow(unused_parens)]
            fn #method_sig {
                #body
            }
        }
    })
}

pub(crate) fn derive_from_segment(input: DeriveInput) -> TokenStream {
    base_from_segment(input, false).unwrap()
}

pub(crate) fn derive_try_from_segment(input: DeriveInput) -> TokenStream {
    base_from_segment(input, true).unwrap()
}
