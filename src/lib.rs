// Inspired by the goose programming language variable size enum 05_shapes.goose

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemEnum, parse_macro_input};

#[proc_macro_attribute]
pub fn variable_enum(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the annotated Rust enum.
    let input = parse_macro_input!(item as ItemEnum);

    let enum_name = &input.ident;
    let vis = &input.vis;

    // Shape -> ShapeDiscriminant
    let discriminant_name = format_ident!("{}Discriminant", enum_name);

    // Collect the variant names, ignoring their payloads.
    let variants = input.variants.iter().map(|variant| &variant.ident);

    // Keep the original enum for this first prototype.
    // Generate a separate enum containing only its variant names.
    let expanded = quote! {
        #input

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #vis enum #discriminant_name {
            #(#variants),*
        }
    };

    expanded.into()
}
