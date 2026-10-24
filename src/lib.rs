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

// Parse boolean flags: #[variable_enum(kind, struct_with_kind, struct_with_prefix = "...", plain_struct)]
struct VariableEnumArgs {
    kind: Option<syn::Ident>, // type given by the user to store the kind
    generate_struct: bool,
    struct_with_kind: bool,
    module_prefix: Option<syn::Path>,
}

impl Parse for VariableEnumArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut kind = None;
        let mut struct_with_kind = false;
        let mut module_prefix = None;

        // This is to generate an error on empty macro call, and have an explicit user choice of plain_struct
        // The user could only select kind, and in this case no struct would be generated alongside it
        let mut plain_struct = false;

        while !input.is_empty() {
            let flag: Ident = input.parse()?;
            match flag.to_string().as_str() {
                "kind" => {
                    input.parse::<Token![=]>()?;

                    let ty = if input.peek(syn::LitStr) {
                        let lit: syn::LitStr = input.parse()?;
                        syn::Ident::new(&lit.value(), lit.span())
                    } else {
                        input.parse::<syn::Ident>()?
                    };

                    match ty.to_string().as_str() {
                        "u8" | "u16" | "u32" | "u64" => {
                            kind = Some(ty);
                        }
                        _ => {
                            return Err(syn::Error::new(
                                ty.span(),
                                "expected one of: u8, u16, u32, u64",
                            ));
                        }
                    }
                }
                "struct_with_kind" => struct_with_kind = true,
                "struct_with_prefix" => {
                    input.parse::<Token![=]>()?;
                    let value: LitStr = input.parse()?;
                    module_prefix = Some(syn::parse_str::<syn::Path>(&value.value())?);
                }
                "plain_struct" => plain_struct = true,
                other => {
                    return Err(syn::Error::new(
                        flag.span(),
                        format!(
                            "unknown flag '{}', expected one of 'kind = \"u8 | u16 | u32 | u64\"', 'struct_with_kind', 'struct_with_prefix = \"...\"', 'plain_struct'",
                            other
                        ),
                    ));
                }
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        if !(struct_with_kind | plain_struct) && kind.is_none() && module_prefix.is_none() {
            return Err(syn::Error::new(
                input.span(),
                "No flag set: expected one of 'kind = \"u8 | u16 | u32 | u64\"', 'struct_with_kind', 'struct_with_prefix = \"...\"', 'plain_struct'",
            ));
        }

        if struct_with_kind && kind.is_none() {
            return Err(syn::Error::new(
                input.span(),
                "struct_with_kind flag requires a kind size. Add kind = \"u8 | u16 | u32 | u64\"",
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
            generate_struct = true;
        }

        if plain_struct && module_prefix.is_some() {
            return Err(syn::Error::new(
                input.span(),
                "struct_with_prefix flag forces the generation of struct. Remove the 'plain_struct' flag",
            ));
        }

        if module_prefix.is_some() {
            generate_struct = true;
        }

        Ok(VariableEnumArgs {
            kind,
            generate_struct,
            struct_with_kind,
            module_prefix,
        })
    }
}

fn generate_kind_enum(
    variants: &Punctuated<Variant, Comma>,
    kind_name: &Ident,
    kind_repr: &Ident,
    visibility: &Visibility,
) -> proc_macro2::TokenStream {
    let kind_variants = variants.iter().enumerate().map(|(index, variant)| {
        let name = &variant.ident;
        let value = syn::LitInt::new(&index.to_string(), variant.ident.span());

        quote! {
            #name = #value
        }
    });

    quote! {
        #[repr(#kind_repr)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #visibility enum #kind_name {
            #(#kind_variants),*
        }
    }
}

fn wrap_in_modules(
    path: &syn::Path,
    content: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    path.segments.iter().rev().fold(content, |inner, segment| {
        let ident = &segment.ident;
        quote! {
            pub mod #ident {
                #inner
            }
        }
    })
}

fn generate_struct_declarations(
    variants: &Punctuated<Variant, Comma>,
    derives: &[syn::Attribute],
    args: &VariableEnumArgs,
) -> proc_macro2::TokenStream {
    let structs = variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        let struct_name = variant_name;

        let fields = match &variant.fields {
            Fields::Named(fields) => {
                let names: Vec<_> = fields
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().unwrap())
                    .collect();
                let types: Vec<_> = fields.named.iter().map(|f| &f.ty).collect();

                quote! {
                    #(
                        pub #names: #types
                    ),*
                }
            }

            Fields::Unnamed(fields) => {
                let names: Vec<_> = (0..fields.unnamed.len())
                    .map(|i| format_ident!("field_{}", i))
                    .collect();
                let types: Vec<_> = fields.unnamed.iter().map(|f| &f.ty).collect();

                quote! {
                    #(
                        pub #names: #types
                    ),*
                }
            }

            Fields::Unit => quote! {},
        };

        let kind_field = if args.struct_with_kind
            && let Some(kind_repr) = &args.kind
        {
            quote! { pub kind: #kind_repr, }
        } else {
            quote! {}
        };

        quote! {
            #[repr(C)]
            #[derive(bytemuck::Pod, bytemuck::Zeroable)]
            #(#derives)*
            pub struct #struct_name {
                #kind_field
                #fields
            }
        }
    });

    let declarations = quote! {
        #(#structs)*
    };

    match &args.module_prefix {
        Some(module_prefix) => {
            let declarations = if args.struct_with_kind {
                quote! {
                    #declarations
                }
            } else {
                declarations
            };

            wrap_in_modules(module_prefix, declarations)
        }
        None => declarations,
    }
}

fn generate_struct_implementations(
    variants: &Punctuated<Variant, Comma>,
    kind_name: &Ident,
    enum_name: &Ident,
    args: &VariableEnumArgs,
) -> proc_macro2::TokenStream {
    let implementations = variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        let struct_name: syn::Path = match &args.module_prefix {
            Some(module_prefix) => {
                let mut path = module_prefix.clone();
                path.segments
                    .push(syn::PathSegment::from(variant_name.clone()));
                path
            }
            None => syn::parse_quote!(#variant_name),
        };

        let (constructor_parameters, field_initializers, enum_pattern, enum_constructor) =
            match &variant.fields {
                Fields::Named(fields) => {
                    let names: Vec<_> = fields
                        .named
                        .iter()
                        .map(|f| f.ident.as_ref().unwrap())
                        .collect();
                    let types: Vec<_> = fields.named.iter().map(|f| &f.ty).collect();

                    (
                        quote! { #(#names: #types),* },
                        quote! { #(#names),* },
                        quote! {
                            #enum_name::#variant_name { #(#names),* }
                        },
                        quote! {
                            #enum_name::#variant_name {
                                #(#names: value.#names),*
                            }
                        },
                    )
                }

                Fields::Unnamed(fields) => {
                    let names: Vec<_> = (0..fields.unnamed.len())
                        .map(|i| format_ident!("field_{}", i))
                        .collect();
                    let types: Vec<_> = fields.unnamed.iter().map(|f| &f.ty).collect();

                    (
                        quote! { #(#names: #types),* },
                        quote! { #(#names),* },
                        quote! {
                            #enum_name::#variant_name(#(#names),*)
                        },
                        quote! {
                            #enum_name::#variant_name(
                                #(value.#names),*
                            )
                        },
                    )
                }

                Fields::Unit => (
                    quote! {},
                    quote! {},
                    quote! { #enum_name::#variant_name },
                    quote! { #enum_name::#variant_name },
                ),
            };

        let (kind_const, kind_initializer) = if args.struct_with_kind
            && let Some(kind_repr) = &args.kind
        {
            (
                quote! {
                    pub const KIND: #kind_name = #kind_name::#variant_name;
                },
                quote! {
                    kind: Self::KIND as #kind_repr,
                },
            )
        } else {
            (quote! {}, quote! {})
        };

        let constructor_initializers = if args.struct_with_kind {
            quote! {
                #kind_initializer
                #field_initializers
            }
        } else {
            quote! {
                #field_initializers
            }
        };

        quote! {
            impl #struct_name {
                #kind_const

                pub fn new(#constructor_parameters) -> Self {
                    Self {
                        #constructor_initializers
                    }
                }
            }

            impl From<#struct_name> for #enum_name {
                fn from(value: #struct_name) -> Self {
                    #enum_constructor
                }
            }

            impl ::core::convert::TryFrom<#enum_name> for #struct_name {
                type Error = #enum_name;

                fn try_from(value: #enum_name) -> Result<Self, Self::Error> {
                    match value {
                        #enum_pattern => Ok(Self {
                            #constructor_initializers
                        }),
                        other => Err(other),
                    }
                }
            }
        }
    });

    quote! {
        #(#implementations)*
    }
}

fn generate_structs(
    variants: &Punctuated<Variant, Comma>,
    kind_name: &Ident,
    enum_name: &Ident,
    derives: &[syn::Attribute],
    args: &VariableEnumArgs,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let declarations = generate_struct_declarations(variants, derives, args);
    let implementations = generate_struct_implementations(variants, kind_name, enum_name, args);

    (declarations, implementations)
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

    let (struct_declarations, struct_implementations) = if args.generate_struct {
        generate_structs(&input.variants, &kind_name, enum_name, &derives, &args)
    } else {
        (quote! {}, quote! {})
    };

    let kind_enum = if let Some(kind_repr) = args.kind {
        generate_kind_enum(&input.variants, &kind_name, &kind_repr, &input.vis)
    } else {
        quote! {}
    };

    let expanded = quote! {
        #input

        #kind_enum

        #struct_declarations
        #struct_implementations
    };

    expanded.into()
}
