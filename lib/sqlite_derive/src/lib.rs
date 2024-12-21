/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! SQLite derive macro's library

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};

/// [FromRow] derive for structs
#[proc_macro_derive(FromRow, attributes(sqlite))]
pub fn from_row_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    // Parse fields and skip fields with #[sqlite(skip)]
    let (fields, has_skipped) = match input.data {
        syn::Data::Struct(data) => {
            let fields_len = data.fields.len();
            let fields: Vec<_> = data
                .fields
                .into_iter()
                .filter(|field| {
                    !field.attrs.iter().any(|attr| {
                        attr.path().is_ident("sqlite")
                            && attr
                                .parse_args::<Ident>()
                                .map_or(false, |ident| ident == "skip")
                    })
                })
                .collect();
            let has_skipped = fields.len() != fields_len;
            (fields, has_skipped)
        }
        _ => panic!("FromRow can only be used on structs"),
    };

    // Generate code
    let columns = fields
        .iter()
        .map(|field| {
            field
                .ident
                .as_ref()
                .expect("Invalid field")
                .to_string()
                .replace("r#", "")
        })
        .collect::<Vec<_>>()
        .join(", ");
    let values = vec!["?"; fields.len()].join(", ");

    let binds = fields.iter().enumerate().map(|(index, field)| {
        let field = field.ident.as_ref().expect("Invalid field");
        let index = index as i32;
        quote! { statement.bind_value(self.#field, #index) }
    });

    let from_rows = fields.iter().enumerate().map(|(index, field)| {
        let field = field.ident.as_ref().expect("Invalid field");
        let field_name = field.to_string().replace("r#", "");
        let index = index as i32;
        quote! { #field: statement.read_value(#index).try_into().unwrap_or_else(|_| panic!(
            "Can't read value of column: {}", #field_name
        )) }
    });
    let from_rows_default = if has_skipped {
        quote! { ..Default::default() }
    } else {
        quote! {}
    };

    TokenStream::from(quote! {
        impl #name {
            pub fn columns() -> &'static str {
                #columns
            }
            pub fn values() -> &'static str {
                #values
            }
        }
        impl sqlite::Bind for #name {
            fn bind(self, statement: &mut sqlite::RawStatement) {
                #( #binds; )*
            }
        }
        impl sqlite::FromRow for #name {
            fn from_row(statement: &mut sqlite::RawStatement) -> Self {
                Self {
                    #( #from_rows, )*
                    #from_rows_default
                }
            }
        }
    })
}

/// [FromValue] derive for enums
#[proc_macro_derive(FromValue)]
pub fn from_value_derive(input: TokenStream) -> TokenStream {
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
            sqlite::Value::Integer(#discriminant) => Ok(#name::#variant_name),
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
            #name::#variant_name => sqlite::Value::Integer(#discriminant),
        }
    });

    TokenStream::from(quote! {
        impl From<#name> for sqlite::Value {
            fn from(value: #name) -> Self {
                match value {
                    #( #to_impls )*
                }
            }
        }
        impl TryFrom<sqlite::Value> for #name {
            type Error = sqlite::ValueError;
            fn try_from(value: sqlite::Value) -> Result<Self, Self::Error> {
                match value {
                    #( #from_impls )*
                    _ => Err(sqlite::ValueError),
                }
            }
        }
    })
}
