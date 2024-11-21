use proc_macro::TokenStream;

#[proc_macro_derive(FromCommitData)]
pub fn derive_from_commit_data(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let result = braid_derive_internals::derive_from_commit_data(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into();

    println!("{}", result);
        
    result
}