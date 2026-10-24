// Inspired by the goose programming language variable size enum 05_shapes.goose

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Fields, Ident, ItemEnum, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

// Parse boolean flags: #[variable_enum(kind, struct_with_kind, prefixed_struct, module_struct, plain_struct)]
struct VariableEnumArgs {
    kind: bool,
    struct_with_kind: bool,
    prefixed_struct: bool,
    module_struct: bool,
    plain_struct: bool,
}

impl Parse for VariableEnumArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut kind = false;
        let mut struct_with_kind = false;
        let mut prefixed_struct = false;
        let mut module_struct = false;
        let mut plain_struct = false;

        while !input.is_empty() {
            let flag: Ident = input.parse()?;
            match flag.to_string().as_str() {
                "kind" => kind = true,
                "struct_with_kind" => struct_with_kind = true,
                "prefixed_struct" => prefixed_struct = true,
                "module_struct" => module_struct = true,
                "plain_struct" => plain_struct = true,
                other => {
                    return Err(syn::Error::new(
                        flag.span(),
                        format!(
                            "unknown flag '{}', expected one of 'kind', 'plain_struct', 'prefixedStruct', 'moduleStruct'",
                            other
                        ),
                    ));
                }
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(VariableEnumArgs {
            kind,
            struct_with_kind,
            prefixed_struct,
            module_struct,
            plain_struct,
        })
    }
}

#[proc_macro_attribute]
pub fn variable_enum(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as VariableEnumArgs);
    let input = parse_macro_input!(item as ItemEnum);

    let enum_name = &input.ident;
    let vis = &input.vis;
    let derives: Vec<_> = input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("derive"))
        .cloned()
        .collect();

    let kind_name = format_ident!("{}Kind", enum_name);

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

        // Generate the kind parts
        let kind_field = if args.kind {
            quote! {
                pub kind: #kind_name,
            }
        } else {
            quote! {}
        };

        let kind_const = if args.kind {
            quote! {
                pub const KIND: #kind_name = #kind_name::#variant_name;
            }
        } else {
            quote! {}
        };

        let kind_initializer = if args.kind {
            quote! {
                kind: Self::KIND,
            }
        } else {
            quote! {}
        };

        quote! {
            #(#derives)*
            pub struct #struct_name {
                #kind_field
                #field_definitions
            }

            impl #struct_name {
                #kind_const

                pub fn new(#constructor_parameters) -> Self {
                    Self {
                        #kind_initializer
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
                            #kind_initializer
                            #field_initializers
                        }),
                        other => Err(other),
                    }
                }
            }
        }
    });

    // Generate the kind enum
    let kind_enum = if args.kind {
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

        let kind_variants = input.variants.iter().enumerate().map(|(index, variant)| {
            let name = &variant.ident;
            let value = syn::LitInt::new(&index.to_string(), variant.ident.span());

            quote! {
                #name = #value
            }
        });

        quote! {
            #[repr(#repr)]
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            #vis enum #kind_name {
                #(#kind_variants),*
            }
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        #input

        #kind_enum

        #(#generated_structs)*
    };

    expanded.into()
}
