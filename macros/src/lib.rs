use proc_macro::TokenStream;

#[macro_use]
extern crate lazy_static;

mod num_getters;

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
