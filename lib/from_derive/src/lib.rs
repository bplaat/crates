/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A [FromEnum] and [FromStruct] derive macro library

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Meta, parse_macro_input};

// MARK: FromEnum
/// [FromEnum] derive
#[proc_macro_derive(FromEnum, attributes(from_enum))]
pub fn from_enum_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let data = match &input.data {
        syn::Data::Enum(data) => data,
        _ => panic!("FromEnum can only be derived for enums"),
    };

    // Parse from_enum other enum name
    let mut other_name = None;
    for attr in &input.attrs {
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
    if let Some(variant) = data
        .variants
        .iter()
        .find(|variant| !matches!(variant.fields, syn::Fields::Unit))
    {
        return syn::Error::new_spanned(variant, "FromEnum only supports unit variants")
            .into_compile_error()
            .into();
    }

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
        impl #impl_generics From<#name #type_generics> for #other_name #type_generics #where_clause {
            fn from(value: #name #type_generics) -> Self {
                match value {
                    #(#variants)*
                }
            }
        }
        impl #impl_generics From<#other_name #type_generics> for #name #type_generics #where_clause {
            fn from(value: #other_name #type_generics) -> Self {
                match value {
                    #(#variants_reverse)*
                }
            }
        }
    })
}

// MARK: FromStruct
/// [FromStruct] derive
#[proc_macro_derive(FromStruct, attributes(from_struct))]
pub fn from_struct_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let data = match &input.data {
        syn::Data::Struct(data) => data,
        _ => panic!("FromStruct can only be derived for structs"),
    };

    if !matches!(data.fields, syn::Fields::Named(_)) {
        return syn::Error::new_spanned(name, "FromStruct only supports structs with named fields")
            .into_compile_error()
            .into();
    }

    // Parse from_struct other struct name
    let mut other_name = None;
    for attr in &input.attrs {
        if attr.path().is_ident("from_struct") {
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
    let other_name = other_name.expect("Missing from_struct attribute");

    // Generate code
    let fields = data.fields.iter().map(|field| {
        let field_name = &field.ident;
        quote! {
            #field_name: value.#field_name.into(),
        }
    });
    let fields_reverse = data.fields.iter().map(|field| {
        let field_name = &field.ident;
        quote! {
            #field_name: value.#field_name.into(),
        }
    });
    TokenStream::from(quote! {
        impl #impl_generics From<#name #type_generics> for #other_name #type_generics #where_clause {
            fn from(value: #name #type_generics) -> Self {
                #other_name {
                    #(#fields)*
                }
            }
        }
        impl #impl_generics From<#other_name #type_generics> for #name #type_generics #where_clause {
            fn from(value: #other_name #type_generics) -> Self {
                #name {
                    #(#fields_reverse)*
                }
            }
        }
    })
}
