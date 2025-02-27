/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A FromEnum derive macro library

#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Meta, parse_macro_input};

/// [FromEnum] derive
#[proc_macro_derive(FromEnum, attributes(from_enum))]
pub fn from_enum_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let data = match input.data {
        syn::Data::Enum(data) => data,
        _ => panic!("FromEnum can only be derived for enums"),
    };

    // Parse from_enum other enum name
    let mut other_name = None;
    for attr in input.attrs {
        if attr.path().is_ident("from_enum") {
            let list = attr
                .parse_args_with(
                    syn::punctuated::Punctuated::<_, syn::token::Comma>::parse_terminated,
                )
                .expect("Invalid attribute");
            for item in list {
                if let Meta::Path(path) = item {
                    other_name = Some(path);
                }
            }
        }
    }
    let other_name = other_name.expect("Missing from_enum attribute");

    // Generate code
    let variants = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        quote! {
            #name::#variant_name => #other_name::#variant_name,
        }
    });
    let variants_reverse = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        quote! {
            #other_name::#variant_name => #name::#variant_name,
        }
    });
    TokenStream::from(quote! {
        impl From<#name> for #other_name {
            fn from(value: #name) -> Self {
                match value {
                    #(#variants)*
                }
            }
        }
        impl From<#other_name> for #name {
            fn from(value: #other_name) -> Self {
                match value {
                    #(#variants_reverse)*
                }
            }
        }
    })
}
