// Inspired by the goose programming language variable size enum 05_shapes.goose

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Fields, ItemEnum, parse_macro_input};

#[proc_macro_attribute]
pub fn variable_enum(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemEnum);

    let enum_name = &input.ident;
    let vis = &input.vis;
    let derives: Vec<_> = input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("derive"))
        .cloned()
        .collect();
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
        let variant_name = &variant.ident;

        let (
            field_definitions,
            constructor_parameters,
            field_initializers,
            enum_pattern,
            enum_constructor,
        ) = match &variant.fields {
            Fields::Named(fields) => {
                let names: Vec<_> = fields
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().unwrap().clone())
                    .collect();
                let types: Vec<_> = fields.named.iter().map(|f| f.ty.clone()).collect();

                let definitions = quote! {
                    #(
                        pub #names: #types
                    ),*
                };

                let parameters = quote! {
                    #(
                        #names: #types
                    ),*
                };

                let initializers = quote! {
                    #(#names),*
                };

                let pattern = quote! {
                    #enum_name::#variant_name {
                        #(#names),*
                    }
                };

                let constructor = quote! {
                    #enum_name::#variant_name {
                        #(#names: value.#names),*
                    }
                };

                (definitions, parameters, initializers, pattern, constructor)
            }

            Fields::Unnamed(fields) => {
                let names: Vec<_> = (0..fields.unnamed.len())
                    .map(|i| format_ident!("field_{}", i))
                    .collect();
                let types: Vec<_> = fields.unnamed.iter().map(|f| f.ty.clone()).collect();

                let definitions = quote! {
                    #(
                        pub #names: #types
                    ),*
                };

                let parameters = quote! {
                    #(
                        #names: #types
                    ),*
                };

                let initializers = quote! {
                    #(#names),*
                };

                let pattern = quote! {
                    #enum_name::#variant_name(#(#names),*)
                };

                let constructor = quote! {
                    #enum_name::#variant_name(
                        #(value.#names),*
                    )
                };

                (definitions, parameters, initializers, pattern, constructor)
            }

            Fields::Unit => (
                quote! {},
                quote! {},
                quote! {},
                quote! { #enum_name::#variant_name },
                quote! { #enum_name::#variant_name },
            ),
        };

        quote! {
            #(#derives)*
            pub struct #struct_name {
                pub kind: #kind_name,
                #field_definitions
            }

            impl #struct_name {
                pub const KIND: #kind_name =
                    #kind_name::#variant_name;

                pub fn new(#constructor_parameters) -> Self {
                    Self {
                        kind: Self::KIND,
                        #field_initializers
                    }
                }
            }

            // Tagged struct -> enum (infallible)
            impl From<#struct_name> for #enum_name {
                fn from(value: #struct_name) -> Self {
                    #enum_constructor
                }
            }

            // Enum -> tagged struct (fallible)
            impl ::core::convert::TryFrom<#enum_name> for #struct_name {
                type Error = #enum_name;

                fn try_from(value: #enum_name) -> Result<Self, Self::Error> {
                    match value {
                        #enum_pattern => Ok(Self {
                            kind: Self::KIND,
                            #field_initializers
                        }),
                        other => Err(other),
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
