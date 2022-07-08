use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn component(metadata: TokenStream, input: TokenStream) -> TokenStream {
    let input: proc_macro2::TokenStream = input.into();
    let expanded = quote! {
        #input
        struct Haha {}

    };
    TokenStream::from(expanded)
}
