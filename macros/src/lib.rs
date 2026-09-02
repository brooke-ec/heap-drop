//! This crate provides a derive macro for the `HeapDrop` trait, providing a function that returns a vector of children.

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

/// Derives the `HeapDrop` trait for a struct or enum, allowing it to be disposed of using a heap-allocated queue.
#[proc_macro_derive(HeapDrop)]
pub fn heap_drop_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let body = match &input.data {
        Data::Struct(s) => {
            let arm = deconstruct_fields(&s.fields);
            quote! { match *self { Self #arm } }
        }
        Data::Enum(e) => {
            let arms = e.variants.iter().map(|v| {
                let arm = deconstruct_fields(&v.fields);
                let variant = &v.ident;
                quote! { Self::#variant #arm }
            });

            quote! { match *self { #(#arms)* } }
        }

        _ => panic!("HeapDrop can only be derived for structs and enums"),
    };

    TokenStream::from(quote! {
        impl #impl_generics ::heap_drop::HeapDrop for #struct_name #ty_generics #where_clause {
            fn into_children(self: Box<Self>) -> Vec<Box<dyn ::heap_drop::HeapDrop>>
            {
                #body
            }
        }
    })
}

/// Emits the tokens to deconstruct fields and collect those which implement `HeapDrop` into a vector.
fn deconstruct_fields(fields: &Fields) -> proc_macro2::TokenStream {
    match fields {
        Fields::Named(fields) => {
            let vars = fields.named.iter().filter_map(|f| f.ident.clone()).collect::<Vec<Ident>>();
            let detections = collect_implementing_vars(&vars);
            quote! { { #(#vars),* , .. } => {#detections} }
        }
        Fields::Unnamed(fields) => {
            let vars = (0..fields.unnamed.len())
                .map(|i| Ident::new(&format!("f{}", i), Span::call_site()))
                .collect::<Vec<Ident>>();
            let detections = collect_implementing_vars(&vars);
            quote! { (#(#vars),*) => {#detections} }
        }
        Fields::Unit => quote! { => vec![] },
    }
}

/// Emits the tokens to collect the values stored at the given identifiers into a vector of those which implement `HeapDrop`.
fn collect_implementing_vars(vars: &Vec<Ident>) -> proc_macro2::TokenStream {
    quote! {
        let mut children: Vec<Box<dyn ::heap_drop::HeapDrop>> = vec![];
        #(if let Some(child) = ::heap_drop::maybe_as_heap_drop!(#vars) { children.push(child); })*
        children
    }
}
