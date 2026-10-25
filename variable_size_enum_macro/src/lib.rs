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

// Parse boolean flags: #[variable_size_enum(kind, struct_with_kind, module = "...", plain_struct)]
struct VariableSizeEnumArgs {
    kind: syn::Ident, // type given by the user to store the kind
    module_prefix: Option<syn::Path>,
}

impl Parse for VariableSizeEnumArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut kind = None;
        let mut module_prefix = None;

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
                "module" => {
                    input.parse::<Token![=]>()?;
                    let value: LitStr = input.parse()?;
                    module_prefix = Some(syn::parse_str::<syn::Path>(&value.value())?);
                }
                other => {
                    return Err(syn::Error::new(
                        flag.span(),
                        format!(
                            "unknown flag '{}', expected one of 'kind = \"u8 | u16 | u32 | u64\"', 'module = \"...\"'",
                            other
                        ),
                    ));
                }
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        if kind.is_none() {
            return Err(syn::Error::new(
                input.span(),
                "Kind flag not set: expected one of 'kind = \"u8 | u16 | u32 | u64\"'",
            ));
        }

        Ok(VariableSizeEnumArgs {
            kind: kind.expect("checked"),
            module_prefix,
        })
    }
}

fn validate_variant_count(
    kind: &Ident,
    variants_len: usize,
    enum_name: &Ident,
) -> Result<(), TokenStream> {
    let max_variants = match kind.to_string().as_str() {
        "u8" => u8::MAX.into(),
        "u16" => u16::MAX.into(),
        "u32" => u32::MAX.into(),
        "u64" => u64::MAX,
        _ => {
            return Err(syn::Error::new_spanned(kind, "Unsupported kind type")
                .to_compile_error()
                .into());
        }
    };

    if variants_len as u64 > max_variants {
        return Err(syn::Error::new_spanned(
            enum_name,
            format!(
                "enum `{}` has {} variants, but `{}` can represent only {} distinct discriminants",
                enum_name,
                variants_len,
                quote::quote!(#kind),
                max_variants,
            ),
        )
        .to_compile_error()
        .into());
    }

    Ok(())
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
    args: &VariableSizeEnumArgs,
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

        let kind_repr = &args.kind;
        let kind_field = quote! { pub kind: #kind_repr, };

        quote! {
            #[repr(C)]
            #[derive(::variable_size_enum::bytemuck::Pod, ::variable_size_enum::bytemuck::Zeroable)]
            #[bytemuck(crate = "variable_size_enum::bytemuck")]
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
            let declarations = quote! {
                #declarations
            };

            wrap_in_modules(module_prefix, declarations)
        }
        None => declarations,
    }
}

fn get_struct_name(args: &VariableSizeEnumArgs, variant_name: &Ident) -> syn::Path {
    match &args.module_prefix {
        Some(module_prefix) => {
            let mut path = module_prefix.clone();
            path.segments
                .push(syn::PathSegment::from(variant_name.clone()));
            path
        }
        None => syn::parse_quote!(#variant_name),
    }
}

fn generate_struct_implementations(
    variants: &Punctuated<Variant, Comma>,
    kind_name: &Ident,
    enum_name: &Ident,
    args: &VariableSizeEnumArgs,
) -> proc_macro2::TokenStream {
    let kind_repr = &args.kind;

    let implementations = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let struct_name: syn::Path = get_struct_name(args, variant_name);

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

        let (kind_const, kind_initializer) = (
            quote! {
                pub const KIND: #kind_name = #kind_name::#variant_name;
            },
            quote! {
                kind: Self::KIND as #kind_repr,
            },
        );

        let constructor_initializers = quote! {
            #kind_initializer
            #field_initializers
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
    args: &VariableSizeEnumArgs,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let declarations = generate_struct_declarations(variants, derives, args);
    let implementations = generate_struct_implementations(variants, kind_name, enum_name, args);

    (declarations, implementations)
}

fn generate_enum_implementation(
    variants: &Punctuated<Variant, Comma>,
    enum_name: &Ident,
    args: &VariableSizeEnumArgs,
) -> proc_macro2::TokenStream {
    let kind_repr = &args.kind;

    let max_size_checks = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let struct_name = get_struct_name(args, variant_name);

        quote! {
            if ::core::mem::size_of::<#struct_name>() > max {
                max = ::core::mem::size_of::<#struct_name>();
            }
        }
    });

    let kind_arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        let enum_pattern = match &variant.fields {
            Fields::Named(_) => {
                quote! {
                    Self::#variant_name { .. }
                }
            }
            Fields::Unnamed(_) => {
                quote! {
                    Self::#variant_name(..)
                }
            }
            Fields::Unit => {
                quote! {
                    Self::#variant_name
                }
            }
        };

        let struct_name: syn::Path = get_struct_name(args, variant_name);

        quote! {
            #enum_pattern => #struct_name::KIND as Self::Kind,
        }
    });

    let from_bytes_arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        let struct_name: syn::Path = get_struct_name(args, variant_name);

        quote! {
            kind if kind == #struct_name::KIND as #kind_repr => {
                debug_assert!(bytes.len() >= ::core::mem::size_of::<#struct_name>(), "Not enough bytes for enum variant");

                let value =
                    ::variable_size_enum::bytemuck::pod_read_unaligned::<#struct_name>(
                        &bytes[..::core::mem::size_of::<#struct_name>()]
                    );

                (value.into(), ::core::mem::size_of::<#struct_name>())
            }
        }
    });

    let to_bytes_arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let struct_name = get_struct_name(args, variant_name);

        let enum_pattern = match &variant.fields {
            Fields::Named(fields) => {
                let names: Vec<_> = fields
                    .named
                    .iter()
                    .map(|field| field.ident.as_ref().unwrap())
                    .collect();

                quote! {
                    Self::#variant_name { #(#names),* }
                }
            }
            Fields::Unnamed(fields) => {
                let names: Vec<_> = (0..fields.unnamed.len())
                    .map(|i| format_ident!("field_{}", i))
                    .collect();

                quote! {
                    Self::#variant_name(#(#names),*)
                }
            }
            Fields::Unit => quote! {
                Self::#variant_name
            },
        };

        let constructor_args = match &variant.fields {
            Fields::Named(fields) => {
                let names: Vec<_> = fields
                    .named
                    .iter()
                    .map(|field| field.ident.as_ref().unwrap())
                    .collect();

                quote! { #(*#names),* }
            }
            Fields::Unnamed(fields) => {
                let names: Vec<_> = (0..fields.unnamed.len())
                    .map(|i| format_ident!("field_{}", i))
                    .collect();

                quote! { #(*#names),* }
            }
            Fields::Unit => quote! {},
        };

        quote! {
            #enum_pattern => {
                let value = #struct_name::new(#constructor_args);
                let written = ::core::mem::size_of::<#struct_name>();

                debug_assert!(out.len() >= written, "output buffer too small");

                // write_unaligned instead of bytemuck::bytes_of + copy_from_slice
                // Could also build the struct in-place ?
                //   Check that the compiler doesn't already optimize this since struct is repr(C) + pod
                unsafe {
                    out.as_mut_ptr()
                        .cast::<u8>()
                        .cast::<#struct_name>()
                        .write_unaligned(value);
                }

                written
            }
        }
    });

    quote! {
        impl ::variable_size_enum::VariableSizeEnum for #enum_name {
            type Kind = #kind_repr;

            const MAX_SIZE: usize = {
                let mut max = 0usize;
                #(#max_size_checks)*
                max
            };

            fn kind(&self) -> Self::Kind {
                match self {
                    #(#kind_arms)*
                }
            }

            fn write_to_uninit(&self, out: &mut [::core::mem::MaybeUninit<u8>]) -> usize {
                match self {
                    #(#to_bytes_arms,)*
                }
            }

            fn from_bytes(bytes: &[u8]) -> (Self, usize) {
                let kind_size = ::core::mem::size_of::<#kind_repr>();

                debug_assert!(bytes.len() >= kind_size, "bytes too short to contain enum kind");

                let kind =
                    ::variable_size_enum::bytemuck::pod_read_unaligned::<#kind_repr>(
                        &bytes[..kind_size]
                    );

                match kind {
                    #(#from_bytes_arms,)*
                    _ => panic!("invalid enum kind"),
                }
            }
        }
    }
}

/// Generates compact, byte-serializable representations for each variant of an enum.
///
/// The macro creates a `#[repr(C)] #[bytemuck::Pod] #[bytemuck::Zeroable]` struct
/// for each enum variant. Each generated struct contains a discriminant field (`kind`)
/// and the fields belonging to that variant.
///
/// This is useful when storing many enum values whose variants have different
/// sizes: each value can be represented using only the bytes needed for its
/// active variant, rather than reserving space for the largest variant.
///
/// # Arguments
///
/// - `kind`: The integer type used to represent the variant discriminant, such
///   as `u8`, `u16`, or `u64`.
/// - `module` (optional): A module path in which to place the generated
///   variant structs. If omitted, the structs are generated in the current
///   scope.
///
/// # Generated items
///
/// For each enum variant, the macro generates a corresponding struct with the
/// same name as the variant. The generated structs provide constructors and
/// conversions used by the serialization implementation.
///
/// The macro also implements [`VariableSizeEnum`] for the original enum. This
/// implementation provides the maximum variant size and methods for encoding
/// and decoding values.
///
/// # Example
///
/// ```rust
/// use variable_size_enum::variable_size_enum;
///
/// #[variable_size_enum(kind = u16)] // u16 so that there is no padding for bytemuck::Pod
/// #[derive(Debug, Copy, Clone, PartialEq)]
/// enum Message {
///     Ping,
///     Data { id: u16, value: u32 },
/// }
///
/// // Each variant has a corresponding generated representation:
/// let ping = Ping::new();
/// let data = Data::new(42, 123);
/// ```
#[proc_macro_attribute]
pub fn variable_size_enum(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as VariableSizeEnumArgs);
    let input = parse_macro_input!(item as ItemEnum);
    let enum_name = &input.ident;

    if let Err(t) = validate_variant_count(&args.kind, input.variants.len(), enum_name) {
        return t;
    }

    let derives: Vec<_> = input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("derive"))
        .cloned()
        .collect();

    let kind_name = format_ident!("{}Kind", enum_name);

    let kind_enum = generate_kind_enum(&input.variants, &kind_name, &args.kind, &input.vis);
    let (struct_declarations, struct_implementations) =
        generate_structs(&input.variants, &kind_name, enum_name, &derives, &args);

    let enum_implementation = generate_enum_implementation(&input.variants, enum_name, &args);

    let expanded = quote! {
        #input

        #kind_enum

        #struct_declarations
        #struct_implementations

        #enum_implementation
    };

    expanded.into()
}
