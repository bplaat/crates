/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! SQLite derive macro's library

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// [FromRow] derive for structs
#[proc_macro_derive(FromRow, attributes(sqlite))]
pub fn from_row_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    // Parse fields and handle #[sqlite(skip)] and #[sqlite(rename = "example")] attributes
    let (fields, has_skipped) = match input.data {
        syn::Data::Struct(data) => {
            let fields_len = data.fields.len();
            let fields: Vec<_> = data
                .fields
                .into_iter()
                .filter_map(|field| {
                    let mut field_name = field
                        .ident
                        .as_ref()
                        .expect("Invalid field")
                        .to_string()
                        .replace("r#", "");
                    for attr in &field.attrs {
                        if attr.path().is_ident("sqlite") {
                            let list = attr
                                .parse_args_with(
                                    syn::punctuated::Punctuated::<_, syn::token::Comma>::parse_terminated,
                                )
                                .expect("Invalid attribute");
                            for meta in list {
                                if let syn::Meta::Path(path) = &meta {
                                    if path.is_ident("skip") {
                                        return None;
                                    }
                                }
                                if let syn::Meta::NameValue(nv) = &meta {
                                    if nv.path.is_ident("rename") {
                                        if let syn::Expr::Lit(syn::ExprLit {
                                            lit: syn::Lit::Str(lit_str),
                                            ..
                                        }) = &nv.value
                                        {
                                            field_name = lit_str.value();
                                        } else {
                                            panic!("Invalid #[sqlite(rename)] value")
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Some((field, field_name))
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
        .map(|(_, field_name)| field_name.clone())
        .collect::<Vec<_>>()
        .join(", ");
    let values = vec!["?"; fields.len()].join(", ");

    let binds = fields.iter().enumerate().map(|(index, (field, _))| {
        let ident = field.ident.as_ref().expect("Invalid field");
        quote! { statement.bind_value(#index as i32, self.#ident) }
    });

    let from_rows = fields
        .iter()
        .enumerate()
        .map(|(index, (field, field_name))| {
            let ident = field.ident.as_ref().expect("Invalid field");
            quote! { #ident: statement.read_value(#index as i32).try_into().unwrap_or_else(|_| panic!(
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
