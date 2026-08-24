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
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let variants = if let syn::Data::Enum(data) = &input.data {
        &data.variants
    } else {
        panic!("FromValue can only be used on enums");
    };

    if let Some(variant) = variants
        .iter()
        .find(|variant| !matches!(variant.fields, syn::Fields::Unit))
    {
        return syn::Error::new_spanned(variant, "FromValue only supports unit variants")
            .into_compile_error()
            .into();
    }

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
        impl #impl_generics From<#name #type_generics> for bsqlite::Value #where_clause {
            fn from(value: #name #type_generics) -> Self {
                match value {
                    #( #to_impls )*
                }
            }
        }
        impl #impl_generics TryFrom<bsqlite::Value> for #name #type_generics #where_clause {
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
