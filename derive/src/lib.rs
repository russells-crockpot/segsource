#![allow(unused_imports, dead_code, unused_variables, unused_mut, unused_macros)]
#![allow(clippy::field_reassign_with_default)]
use proc_macro::TokenStream;
use syn::parse_macro_input;

extern crate alloc;

mod from_seg;
pub(crate) mod util;

#[macro_use]
extern crate here;

#[proc_macro_derive(FromSegment, attributes(from_seg))]
pub fn from_segment(stream: TokenStream) -> TokenStream {
    TokenStream::from(from_seg::derive_from_segment(parse_macro_input!(stream)))
}

#[proc_macro_derive(TryFromSegment, attributes(from_seg))]
pub fn try_from_segment(stream: TokenStream) -> TokenStream {
    TokenStream::from(from_seg::derive_try_from_segment(parse_macro_input!(
        stream
    )))
}
