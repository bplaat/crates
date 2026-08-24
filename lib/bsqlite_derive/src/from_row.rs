/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub(crate) fn from_row_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    // Parse fields and handle #[sqlite(skip)] and #[sqlite(rename = "example")] attributes
    let (fields, has_skipped) = match &input.data {
        syn::Data::Struct(data) => {
            if !matches!(data.fields, syn::Fields::Named(_)) {
                return syn::Error::new_spanned(
                    name,
                    "FromRow only supports structs with named fields",
                )
                .into_compile_error()
                .into();
            }
            let fields_len = data.fields.len();
            let fields = data
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
                    Some((field.clone(), field_name))
                })
                .collect::<Vec<_>>();
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
        let index = index as i32;
        let ident = field.ident.as_ref().expect("Invalid field");
        quote! { statement.bind_value(#index, self.#ident.into())?; }
    });

    let from_rows = fields
        .iter()
        .enumerate()
        .map(|(index, (field, field_name))| {
            let index = index as i32;
            let ident = field.ident.as_ref().expect("Invalid field");
            quote! { #ident: statement.column_value(#index).try_into().map_err(|_| bsqlite::ValueError::new(format!(
                "Can't get value of column: {}", #field_name
            )))? }
        });
    let from_rows_default = if has_skipped {
        quote! { ..Default::default() }
    } else {
        quote! {}
    };

    TokenStream::from(quote! {
        impl #impl_generics #name #type_generics #where_clause {
            pub const fn columns() -> &'static str {
                #columns
            }
            pub const fn values() -> &'static str {
                #values
            }
        }
        impl #impl_generics bsqlite::Bind for #name #type_generics #where_clause {
            fn bind(self, statement: &mut bsqlite::RawStatement) -> Result<(), bsqlite::StatementError> {
                #( #binds )*
                Ok(())
            }
        }
        impl #impl_generics bsqlite::FromRow for #name #type_generics #where_clause {
            fn from_row(statement: &mut bsqlite::RawStatement) -> Result<Self, bsqlite::ValueError> {
                Ok(Self {
                    #( #from_rows, )*
                    #from_rows_default
                })
            }
        }
    })
}
