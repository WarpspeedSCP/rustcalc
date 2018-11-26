extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use crate::proc_macro::TokenStream;

#[proc_macro_derive(EncapsulatingNode)]
pub fn encapsulating_node_derive(input: TokenStream) -> TokenStream {
    // Construct a represntation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_encapsulating_node(&ast)
}

fn impl_encapsulating_node(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    (quote! {
        impl NodeT for #name {}
        impl EncapsulatingNode for #name {}
    }).into()
}