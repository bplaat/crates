/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Derive macros for the berde library.

#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Field, parse_macro_input};

/// [Serialize] derive
#[proc_macro_derive(Serialize, attributes(berde))]
pub fn serialize_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let fields = parse_fields(&input);
    let name = input.ident;

    let num_fields = fields.len();

    let serialize_fields = fields.iter().map(|(field, field_name)| {
        let ident = field.ident.as_ref().expect("Invalid field");
        quote! {
            serializer.serialize_field(#field_name, &self.#ident);
        }
    });

    TokenStream::from(quote! {
        impl Serialize for #name {
            fn serialize(&self, serializer: &mut dyn Serializer) {
                serializer.serialize_start_struct(stringify!(#name), #num_fields);
                #(#serialize_fields)*
                serializer.serialize_end_struct();
            }
        }
    })
}

/// [Deserialize] derive
#[proc_macro_derive(Deserialize, attributes(berde))]
pub fn deserialize_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let fields = parse_fields(&input);
    let name = input.ident;

    let init_fields = fields.iter().map(|(field, _)| {
        let ident = field.ident.as_ref().expect("Invalid field");
        quote! {
            let mut #ident = None;
        }
    });

    let deserialize_fields = fields.iter().map(|(field, field_name)| {
        let ident = field.ident.as_ref().expect("Invalid field");
        quote! {
            #field_name => #ident = Some(Deserialize::deserialize(value_deserializer)?),
        }
    });

    let construct_fields = fields.iter().map(|(field, _)| {
        let ident = field.ident.as_ref().expect("Invalid field");
        quote! {
            #ident: #ident.ok_or(DeserializeError)?,
        }
    });

    TokenStream::from(quote! {
        impl Deserialize for #name {
            fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self, DeserializeError> {
                #(#init_fields)*
                deserializer.deserialize_start_struct(stringify!(#name))?;
                while let Some((field, value_deserializer)) = deserializer.deserialize_field()? {
                    match field {
                        #(#deserialize_fields)*
                        _ => {}
                    }
                }
                deserializer.deserialize_end_struct()?;
                Ok(#name {
                    #(#construct_fields)*
                })
            }
        }
    })
}

fn parse_fields(input: &DeriveInput) -> Vec<(Field, String)> {
    match &input.data {
        syn::Data::Struct(data) => {
            data
                .fields
                .iter()
                .filter_map(|field| {
                    let mut field_name = field
                        .ident
                        .as_ref()
                        .expect("Invalid field")
                        .to_string()
                        .replace("r#", "");
                    for attr in &field.attrs {
                        if attr.path().is_ident("berde") {
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
                                            panic!("Invalid #[berde(rename)] value")
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Some((field.clone(), field_name))
                })
                .collect::<Vec<_>>()
        }
        _ => panic!("This derive macro can only be used on structs"),
    }
}
