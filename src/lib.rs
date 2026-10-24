// Inspired by the goose programming language variable size enum 05_shapes.goose

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Fields, Ident, ItemEnum, LitStr, Token, Variant, Visibility,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token::Comma,
};

// Parse boolean flags: #[variable_enum(kind, struct_with_kind, prefixed_struct, module_struct)]
struct VariableEnumArgs {
    kind: bool,
    generate_struct: bool,
    struct_with_kind: bool,
    prefix: Option<String>,
    module_struct: bool,
}

impl Parse for VariableEnumArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut kind = false;
        let mut struct_with_kind = false;
        let mut prefix = None;
        let mut module_struct = false;

        // This is to generate an error on empty macro call, and have an explicit user choice of plain_struct
        // The user could only select kind, and in this case no struct would be generated alongside it
        let mut plain_struct = false;

        while !input.is_empty() {
            let flag: Ident = input.parse()?;
            match flag.to_string().as_str() {
                "kind" => kind = true,
                "struct_with_kind" => struct_with_kind = true,
                "struct_with_prefix" => {
                    input.parse::<Token![=]>()?;
                    let value: LitStr = input.parse()?;
                    prefix = Some(value.value());
                }
                "module_struct" => module_struct = true,
                "plain_struct" => plain_struct = true,
                other => {
                    return Err(syn::Error::new(
                        flag.span(),
                        format!(
                            "unknown flag '{}', expected one of 'kind', 'struct_with_kind', 'struct_with_prefix = \"...\"', 'moduleStruct', 'plain_struct'",
                            other
                        ),
                    ));
                }
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        if !(kind | struct_with_kind | module_struct | plain_struct) || prefix.is_some() {
            return Err(syn::Error::new(
                input.span(),
                "No flag set: expected one of 'kind', 'struct_with_kind', 'struct_with_prefix = \"...\"', 'moduleStruct', 'plain_struct'",
            ));
        }

        if kind && struct_with_kind {
            return Err(syn::Error::new(
                input.span(),
                "struct_with_kind flag forces the generation of kind. Remove the 'kind' flag",
            ));
        }
        if plain_struct && struct_with_kind {
            return Err(syn::Error::new(
                input.span(),
                "struct_with_kind flag forces the generation of struct. Remove the 'plain_struct' flag",
            ));
        }

        let mut generate_struct = plain_struct;
        // Force the generation of kind and struct if flag request it
        if struct_with_kind {
            kind = true;
            generate_struct = true;
        }

        if plain_struct && module_struct {
            return Err(syn::Error::new(
                input.span(),
                "module_struct flag forces the generation of struct. Remove the 'plain_struct' flag",
            ));
        }

        if module_struct {
            generate_struct = true;
        }

        if plain_struct && prefix.is_some() {
            return Err(syn::Error::new(
                input.span(),
                "struct_with_prefix flag forces the generation of struct. Remove the 'plain_struct' flag",
            ));
        }

        if prefix.is_some() {
            generate_struct = true;
        }

        Ok(VariableEnumArgs {
            kind,
            generate_struct,
            struct_with_kind,
            prefix,
            module_struct,
        })
    }
}

fn generate_kind_enum_parts(
    kind_name: &Ident,
    variant_name: &Ident,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
) {
    let kind_field = quote! {
        pub kind: #kind_name,
    };

    let kind_const = quote! {
        pub const KIND: #kind_name = #kind_name::#variant_name;
    };

    let kind_initializer = quote! {
        kind: Self::KIND,
    };

    (kind_field, kind_const, kind_initializer)
}

fn generate_kind_enum(
    variants: &Punctuated<Variant, Comma>,
    kind_name: &Ident,
    visibility: &Visibility,
) -> proc_macro2::TokenStream {
    let variant_count = variants.len();
    let repr = if variant_count <= (u8::MAX as usize) + 1 {
        quote! { u8 }
    } else if variant_count <= (u16::MAX as usize) + 1 {
        quote! { u16 }
    } else if variant_count <= (u32::MAX as usize) + 1 {
        quote! { u32 }
    } else {
        quote! { u64 }
    };

    let kind_variants = variants.iter().enumerate().map(|(index, variant)| {
        let name = &variant.ident;
        let value = syn::LitInt::new(&index.to_string(), variant.ident.span());

        quote! {
            #name = #value
        }
    });

    quote! {
        #[repr(#repr)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #visibility enum #kind_name {
            #(#kind_variants),*
        }
    }
}

fn generate_structs(
    variants: &Punctuated<Variant, Comma>,
    kind_name: &Ident,
    enum_name: &Ident,
    derives: &Vec<syn::Attribute>,
    args: &VariableEnumArgs,
) -> Vec<proc_macro2::TokenStream> {
    variants
        .iter()
        .map(|variant| {
            let variant_name = &variant.ident;
            let struct_name = match &args.prefix {
                Some(prefix) => &format_ident!("{}{}", prefix, variant_name),
                None => variant_name,
            };

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
            let (kind_field, kind_const, kind_initializer) = if args.struct_with_kind {
                generate_kind_enum_parts(kind_name, variant_name)
            } else {
                (quote! {}, quote! {}, quote! {})
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
        })
        .collect()
}

#[proc_macro_attribute]
pub fn variable_enum(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as VariableEnumArgs);
    let input = parse_macro_input!(item as ItemEnum);

    let enum_name = &input.ident;
    let derives: Vec<_> = input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("derive"))
        .cloned()
        .collect();

    let kind_name = format_ident!("{}Kind", enum_name);

    let generated_structs = if args.generate_struct {
        generate_structs(&input.variants, &kind_name, enum_name, &derives, &args)
    } else {
        vec![]
    };

    let kind_enum = if args.kind {
        generate_kind_enum(&input.variants, &kind_name, &input.vis)
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
