use proc_macro::TokenStream;
use syn::parse_macro_input;

mod from_seg;
pub(crate) mod util;

#[proc_macro_derive(FromSegment, attributes(from_seg, from_item_type))]
pub fn from_segment(stream: TokenStream) -> TokenStream {
    TokenStream::from(from_seg::derive_from_segment(parse_macro_input!(stream)))
}

#[proc_macro_derive(
    TryFromSegment,
    attributes(try_from_seg, try_from_item_type, try_from_error)
)]
pub fn try_from_segment(stream: TokenStream) -> TokenStream {
    TokenStream::from(from_seg::derive_try_from_segment(parse_macro_input!(
        stream
    )))
}
