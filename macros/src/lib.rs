use proc_macro::TokenStream;
use syn::parse_macro_input;

#[macro_use]
extern crate lazy_static;

mod from_seg;
mod num_getters;
pub(crate) mod util;

#[proc_macro]
pub fn impl_next_methods(stream: TokenStream) -> TokenStream {
    num_getters::impl_next_methods(stream)
}

#[proc_macro]
pub fn impl_at_methods(stream: TokenStream) -> TokenStream {
    num_getters::impl_at_methods(stream)
}

#[proc_macro]
pub fn make_number_methods(stream: TokenStream) -> TokenStream {
    num_getters::make_number_methods(stream)
}

#[proc_macro]
pub fn for_each_number(stream: TokenStream) -> TokenStream {
    num_getters::for_each_number(stream)
}

#[proc_macro_derive(FromSegment, attributes(from_seg, from_item_type))]
pub fn from_bytes_segment(stream: TokenStream) -> TokenStream {
    TokenStream::from(from_seg::derive_from_segment(parse_macro_input!(stream)))
}

#[proc_macro_derive(
    TryFromSegment,
    attributes(try_from_seg, try_from_item_type, try_from_error)
)]
pub fn try_from_bytes_segment(stream: TokenStream) -> TokenStream {
    TokenStream::from(from_seg::derive_try_from_segment(parse_macro_input!(
        stream
    )))
}
