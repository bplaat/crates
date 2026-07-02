/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub(crate) fn from_value_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let variants = if let syn::Data::Enum(data) = input.data {
        data.variants
    } else {
        panic!("FromValue can only be used on enums");
    };

    let from_impls = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let discriminant = if let Some((_, expr)) = &variant.discriminant {
            quote! { #expr }
        } else {
            panic!("Enum variants must have discriminants");
        };
        quote! {
            bsqlite::Value::Integer(#discriminant) => Ok(#name::#variant_name),
        }
    });

    let to_impls = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let discriminant = if let Some((_, expr)) = &variant.discriminant {
            quote! { #expr }
        } else {
            panic!("Enum variants must have discriminants");
        };
        quote! {
            #name::#variant_name => bsqlite::Value::Integer(#discriminant),
        }
    });

    TokenStream::from(quote! {
        impl From<#name> for bsqlite::Value {
            fn from(value: #name) -> Self {
                match value {
                    #( #to_impls )*
                }
            }
        }
        impl TryFrom<bsqlite::Value> for #name {
            type Error = bsqlite::ValueError;
            fn try_from(value: bsqlite::Value) -> Result<Self, Self::Error> {
                match value {
                    #( #from_impls )*
                    _ => Err(bsqlite::ValueError::new("invalid enum variant")),
                }
            }
        }
    })
}
