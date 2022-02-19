use pmhelp::{exts::ParseBufferExt as _, from_parens, parse::parse_stream::comma_separated};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    Data, DataEnum, DataStruct, DeriveInput, Fields, FieldsNamed, FieldsUnnamed, Ident, Path,
    Token, Type, TypePath,
};

mod kw {
    syn::custom_keyword!(error);
    //TODO add support for them to have generics
    //TODO support naming/unpacking
    syn::custom_keyword!(also_needs);
    syn::custom_keyword!(item);
    syn::custom_keyword!(preparse);
    syn::custom_keyword!(postparse);
}

pub struct AlsoNeedsEntry {
    ident: Ident,
    ty: Type,
}

impl Parse for AlsoNeedsEntry {
    fn parse(stream: ParseStream) -> Result<Self> {
        Ok(Self {
            ident: stream.parse()?,
            ty: {
                stream.parse::<Token![:]>()?;
                stream.parse()?
            },
        })
    }
}

#[derive(Default)]
pub struct AlsoNeeds {
    segment_generics: Option<TokenStream>,
    additional_types: Option<Punctuated<AlsoNeedsEntry, Token![,]>>,
}

impl AlsoNeeds {
    pub fn get_default_args() -> TokenStream {
        quote! {segment}
    }

    pub fn set_segment_generics(&mut self, segment_generics: TokenStream) {
        if self.segment_generics.is_some() {
            panic!("Attempted to set segment_generics twice!")
        }
        self.segment_generics = Some(segment_generics);
    }

    pub fn get_segment_type(&self) -> TokenStream {
        let segment_generics = self
            .segment_generics
            .as_ref()
            .expect("segment_generics has not been set!");
        quote! {&::segsource::Segment<#segment_generics>}
    }

    pub fn get_type(&self) -> TokenStream {
        let default = self.get_segment_type();
        if let Some(also_needs) = self.additional_types.as_ref() {
            if also_needs.is_empty() {
                default
            } else {
                let types: Vec<&Type> = also_needs.iter().map(|an| &an.ty).collect();
                quote! {(#(#types,)* #default)}
            }
        } else {
            default
        }
    }

    pub fn get_args(&self) -> TokenStream {
        let default = Self::get_default_args();
        if let Some(also_needs) = self.additional_types.as_ref() {
            if also_needs.is_empty() {
                default
            } else {
                let idents: Vec<&Ident> = also_needs.iter().map(|an| &an.ident).collect();
                quote! {(#(#idents,)* #default)}
            }
        } else {
            default
        }
    }
}

impl Parse for AlsoNeeds {
    #[inline]
    fn parse(stream: ParseStream) -> Result<Self> {
        Ok(Self {
            segment_generics: None,
            additional_types: Some(comma_separated(stream)?),
        })
    }
}

enum FromSegEntry {
    Error(Type),
    Item(Type),
    AlsoNeeds(AlsoNeeds),
    Preparse(Path),
    Postparse(Path),
}

impl FromSegEntry {
    fn apply(self, info: &mut FromSegInfo) {
        match self {
            Self::Item(ty) => info.item_type = ty,
            Self::Error(ty) => info.error_type = Some(ty),
            Self::AlsoNeeds(also_needs) => info.also_needs = also_needs,
            Self::Preparse(path) => info.preparse = Some(path),
            Self::Postparse(path) => info.postparse = Some(path),
        }
    }
}

impl Parse for FromSegEntry {
    fn parse(stream: ParseStream) -> Result<Self> {
        if stream.peek_and_consume(kw::error) {
            Ok(Self::Error(from_parens!(stream).parse()?))
        } else if stream.peek_and_consume(kw::preparse) {
            Ok(Self::Preparse(from_parens!(stream).parse()?))
        } else if stream.peek_and_consume(kw::postparse) {
            Ok(Self::Postparse(from_parens!(stream).parse()?))
        } else if stream.peek_and_consume(kw::item) {
            Ok(Self::Item(from_parens!(stream).parse()?))
        } else if stream.peek_and_consume(kw::also_needs) {
            Ok(Self::AlsoNeeds(from_parens!(stream).parse()?))
        } else {
            Err(stream.error("Invalid option to top level from_seg attribute"))
        }
    }
}

pub struct FromSegInfo {
    pub error_type: Option<Type>,
    pub item_type: Type,
    pub also_needs: AlsoNeeds,
    pub preparse: Option<Path>,
    pub postparse: Option<Path>,
}

impl Default for FromSegInfo {
    fn default() -> Self {
        Self {
            error_type: None,
            item_type: Type::Path(TypePath {
                qself: None,
                path: format_ident!("u8").into(),
            }),
            also_needs: Default::default(),
            preparse: None,
            postparse: None,
        }
    }
}

impl Parse for FromSegInfo {
    fn parse(stream: ParseStream) -> Result<Self> {
        let mut info = Self::default();
        comma_separated::<FromSegEntry>(stream)?
            .into_iter()
            .for_each(|e| e.apply(&mut info));
        Ok(info)
    }
}
