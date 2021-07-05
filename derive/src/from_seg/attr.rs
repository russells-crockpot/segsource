use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream, Result as ParseResult},
    punctuated::Punctuated,
    Expr, ExprCall, ExprField, ExprLit, ExprPath, Field, Ident, Lit, LitInt, Path, Token, Type,
};

#[derive(PartialEq)]
enum TryOption {
    Unwrap = 1,
    Try = 2,
    Nothing = 3,
    Default = 4,
}

impl Default for TryOption {
    fn default() -> Self {
        TryOption::Default
    }
}

enum FromOption {
    Default,
    Type(Box<Type>),
}

impl Parse for FromOption {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        Ok(Self::Type(stream.parse()?))
    }
}

enum MapEachOption {
    Default,
    Expr(Box<Expr>),
}

impl Parse for MapEachOption {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        Ok(Self::Expr(stream.parse()?))
    }
}

enum SizeOption {
    FieldName(Ident),
    Field(ExprField),
    Constant(LitInt),
    Call(ExprCall),
}

impl Parse for SizeOption {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        match Expr::parse(stream)? {
            Expr::Lit(ExprLit {
                lit: Lit::Int(int), ..
            }) => Ok(Self::Constant(int)),
            Expr::Path(ExprPath { path, .. }) => {
                if let Some(ident) = path.get_ident() {
                    Ok(Self::FieldName(ident.clone()))
                } else {
                    panic!("Invalid value {} for size option.", path.to_token_stream())
                }
            }
            Expr::Field(field) => Ok(Self::Field(field)),
            Expr::Call(call) => Ok(Self::Call(call)),
            other => panic!("Invalid value {} for size option.", other.to_token_stream()),
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
        }
    }
}

enum FromSegEntry {
    Skip,
    MapEach(MapEachOption),
    FromIter,
    From(FromOption),
    TryFrom(FromOption),
    Size(SizeOption),
    DefaultValue(Expr),
    Use(Path),
    Try(TryOption),
    If(Expr),
}

fn get_attr_value<P: Parse>(stream: ParseStream) -> ParseResult<P> {
    stream.parse::<Token![=]>()?;
    stream.parse::<P>()
}

impl FromSegEntry {
    fn assign(self, from_seg: &mut FromSegField) {
        match self {
            Self::Try(value) => from_seg.try_ = value,
            Self::From(value) => from_seg.from = Some(value),
            Self::TryFrom(value) => from_seg.try_from = Some(value),
            Self::Size(value) => from_seg.size = Some(value),
            Self::DefaultValue(value) => from_seg.default_value = Some(value),
            Self::Use(value) => from_seg.use_ = Some(value),
            Self::If(value) => from_seg.if_ = Some(value),
            Self::MapEach(value) => {
                from_seg.from_iter = true;
                from_seg.map_each = Some(value);
            }
            Self::Skip => from_seg.skip = true,
            Self::FromIter => from_seg.from_iter = true,
        }
    }
}

impl Parse for FromSegEntry {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        if stream.peek(Token![try]) {
            stream.parse::<Token![try]>()?;
            if stream.peek(Token![try]) {
                stream.parse::<Token![try]>()?;
                Ok(Self::Try(TryOption::Try))
            } else {
                let value: Ident = get_attr_value(stream)?;
                if value == "unwrap" {
                    Ok(Self::Try(TryOption::Unwrap))
                } else if value == "nothing" {
                    Ok(Self::Try(TryOption::Nothing))
                } else {
                    panic!("Invalid try option: {}", value);
                }
            }
        } else if stream.peek(Token![use]) {
            stream.parse::<Token![use]>()?;
            Ok(Self::Use(stream.parse()?))
        } else if stream.peek(Token![if]) {
            stream.parse::<Token![if]>()?;
            Ok(Self::If(stream.parse()?))
        } else {
            let ident: Ident = stream.parse()?;
            if ident == "skip" {
                Ok(Self::Skip)
            } else if ident == "from_iter" {
                Ok(Self::FromIter)
            } else if ident == "map_each" {
                if stream.peek(Token![=]) {
                    Ok(Self::MapEach(get_attr_value(stream)?))
                } else {
                    Ok(Self::MapEach(MapEachOption::Default))
                }
            } else if ident == "size" {
                Ok(Self::Size(stream.parse()?))
            } else if ident == "default_value" {
                Ok(Self::DefaultValue(stream.parse()?))
            } else if ident == "from" {
                if stream.peek(Token![=]) {
                    Ok(Self::From(get_attr_value(stream)?))
                } else {
                    Ok(Self::From(FromOption::Default))
                }
            } else if ident == "try_from" {
                if stream.peek(Token![=]) {
                    Ok(Self::TryFrom(get_attr_value(stream)?))
                } else {
                    Ok(Self::TryFrom(FromOption::Default))
                }
            } else {
                panic!("Invalid from_seg name: {}", stream)
            }
        }
    }
}

struct FromSegEntries(Punctuated<FromSegEntry, Token![,]>);

impl FromSegEntries {
    fn into_inner(self) -> Punctuated<FromSegEntry, Token![,]> {
        self.0
    }
}

impl Parse for FromSegEntries {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        let content;
        let meta_parser = Punctuated::<FromSegEntry, Token![,]>::parse_separated_nonempty;
        let _ = parenthesized!(content in stream);
        Ok(Self(meta_parser(&content)?))
    }
}

pub struct FromSegField {
    tmp_var: Ident,
    ty: Type,
    generating_try_from: bool,
    segment_type: TokenStream,
    use_: Option<Path>,
    try_from: Option<FromOption>,
    from: Option<FromOption>,
    size: Option<SizeOption>,
    try_: TryOption,
    skip: bool,
    map_each: Option<MapEachOption>,
    from_iter: bool,
    if_: Option<Expr>,
    default_value: Option<Expr>,
}

impl FromSegField {
    fn with_defaults(
        tmp_var: Ident,
        ty: Type,
        generating_try_from: bool,
        segment_type: TokenStream,
    ) -> Self {
        Self {
            tmp_var,
            ty,
            generating_try_from,
            segment_type,
            use_: None,
            try_from: None,
            from: None,
            size: None,
            try_: TryOption::Default,
            skip: false,
            map_each: None,
            from_iter: false,
            if_: None,
            default_value: None,
        }
    }

    fn get_default_value(&self) -> TokenStream {
        self.default_value
            .as_ref()
            .map(ToTokens::to_token_stream)
            .unwrap_or(quote! {Default::default()})
    }

    fn get_assign_value(&self) -> TokenStream {
        let base = if self.skip {
            self.get_default_value()
        } else if self.from_iter || self.map_each.is_some() {
            let map_each = self.get_map_each();
            let size = &self.size;
            let suffix = self.get_try_suffix_ignore_none();
            quote! {
                segment.get_n_as_slice(#size)#suffix
                    #map_each
                    .collect()
            }
        } else {
            self.get_simple_assign_val(quote! {segment})
        };
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
        let segment_type = &self.segment_type;
        if let Some(use_) = &self.use_ {
            quote! {#use_(#value)#suffix}
        } else if let Some(FromOption::Default) = &self.from {
            let ty = &self.ty;
            quote! {<#ty as ::std::convert::From<#segment_type>>::from(#value)}
        } else if let Some(FromOption::Type(ty)) = &self.from {
            quote! {<#ty as ::std::convert::From<#segment_type>>::from(#value)}
        } else if let Some(FromOption::Default) = &self.try_from {
            let ty = &self.ty;
            quote! {<#ty as ::std::convert::TryFrom<#segment_type>>::try_from(#value)#suffix}
        } else if let Some(FromOption::Type(ty)) = &self.try_from {
            quote! {<#ty as ::std::convert::TryFrom<#segment_type>>::try_from(#value)#suffix}
        } else if self.generating_try_from {
            let ty = &self.ty;
            quote! {<#ty as ::std::convert::TryFrom<#segment_type>>::try_from(#value)#suffix}
        } else {
            let ty = &self.ty;
            quote! {<#ty as ::std::convert::From<#segment_type>>::from(#value)}
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
            TryOption::Default | TryOption::Nothing if self.generating_try_from => quote! {?},
            TryOption::Unwrap => quote! {.unwrap()},
            TryOption::Default | TryOption::Nothing if !self.generating_try_from => {
                quote! {.unwrap()}
            }
            _ => unreachable!(),
        }
    }

    fn get_try_suffix(&self) -> Option<TokenStream> {
        match self.try_ {
            TryOption::Nothing => None,
            _ => Some(self.get_try_suffix_ignore_none()),
        }
    }

    pub fn tmp_var(&self) -> Ident {
        self.tmp_var.clone()
    }
}

impl From<(usize, (Field, bool, TokenStream))> for FromSegField {
    fn from(
        (idx, (field, generating_try_from, segment_type)): (usize, (Field, bool, TokenStream)),
    ) -> Self {
        let tmp_var = field
            .ident
            .clone()
            .unwrap_or(quote::format_ident!("tmp_{}", idx));
        let mut me =
            FromSegField::with_defaults(tmp_var, field.ty, generating_try_from, segment_type);
        let attr_name = if generating_try_from {
            "try_from_seg"
        } else {
            "from_seg"
        };
        for attr in field.attrs {
            if attr.path.is_ident(attr_name) {
                syn::parse2::<FromSegEntries>(attr.tokens)
                    .map(FromSegEntries::into_inner)
                    .unwrap()
                    .into_iter()
                    .for_each(|entry| entry.assign(&mut me));
                break;
            }
        }
        if me.default_value.is_some() && (me.if_.is_none() || !me.skip) {
            panic!("Cannot specify default_value without an if condition or skip!")
        }
        if (me.map_each.is_some() || me.from_iter) && me.size.is_none() {
            panic!("A size is needed for repeated values!")
        }
        me
    }
}

impl ToTokens for FromSegField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let tmp_var = &self.tmp_var;
        let lhs = self.get_assign_value();
        let result = quote! { let #tmp_var = #lhs; };
        tokens.extend(result);
    }
}
