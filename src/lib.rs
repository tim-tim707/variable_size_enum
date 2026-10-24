// Inspired by the goose programming language variable size enum 05_shapes.goose

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Fields, ItemEnum, parse_macro_input};

#[proc_macro_attribute]
pub fn variable_enum(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemEnum);

    let enum_name = &input.ident;
    let vis = &input.vis;
    let variant_count = input.variants.len();

    let repr = if variant_count <= (u8::MAX as usize) + 1 {
        quote! { u8 }
    } else if variant_count <= (u16::MAX as usize) + 1 {
        quote! { u16 }
    } else if variant_count <= (u32::MAX as usize) + 1 {
        quote! { u32 }
    } else {
        quote! { u64 }
    };

    let kind_name = format_ident!("{}Kind", enum_name);

    let kind_variants = input.variants.iter().enumerate().map(|(index, variant)| {
        let name = &variant.ident;
        let value = syn::LitInt::new(&index.to_string(), variant.ident.span());

        quote! {
            #name = #value
        }
    });

    let generated_structs = input.variants.iter().map(|variant| {
        let struct_name = &variant.ident;
        let kind_variant = &variant.ident;

        let (field_definitions, constructor_parameters, field_initializers) = match &variant.fields
        {
            Fields::Named(fields) => {
                let definitions = fields.named.iter().map(|field| {
                    let name = field.ident.as_ref().unwrap();
                    let ty = &field.ty;
                    quote! {
                        #vis #name: #ty
                    }
                });

                let parameters = fields.named.iter().map(|field| {
                    let name = field.ident.as_ref().unwrap();
                    let ty = &field.ty;
                    quote! {
                        #name: #ty
                    }
                });

                let initializers = fields.named.iter().map(|field| {
                    let name = field.ident.as_ref().unwrap();
                    quote! {
                        #name
                    }
                });

                (
                    quote! { #(#definitions,)* },
                    quote! { #(#parameters,)* },
                    quote! { #(#initializers,)* },
                )
            }

            Fields::Unnamed(fields) => {
                let definitions = fields.unnamed.iter().enumerate().map(|(i, field)| {
                    let name = format_ident!("field_{}", i);
                    let ty = &field.ty;
                    quote! {
                        #vis #name: #ty
                    }
                });

                let parameters = fields.unnamed.iter().enumerate().map(|(i, field)| {
                    let name = format_ident!("field_{}", i);
                    let ty = &field.ty;
                    quote! {
                        #name: #ty
                    }
                });

                let initializers = fields.unnamed.iter().enumerate().map(|(i, _)| {
                    let name = format_ident!("field_{}", i);
                    quote! {
                        #name
                    }
                });

                (
                    quote! { #(#definitions,)* },
                    quote! { #(#parameters,)* },
                    quote! { #(#initializers,)* },
                )
            }

            Fields::Unit => (quote! {}, quote! {}, quote! {}),
        };

        quote! {
            #vis struct #struct_name {
                #vis kind: #kind_name,
                #field_definitions
            }

            impl #struct_name {
                pub const KIND: #kind_name =
                    #kind_name::#kind_variant;

                pub fn new(#constructor_parameters) -> Self {
                    Self {
                        kind: Self::KIND,
                        #field_initializers
                    }
                }
            }
        }
    });

    let expanded = quote! {
        #input

        #[repr(#repr)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #vis enum #kind_name {
            #(#kind_variants),*
        }

        #(#generated_structs)*
    };

    expanded.into()
}
