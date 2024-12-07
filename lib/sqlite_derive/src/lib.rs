/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};

#[proc_macro_derive(FromRow, attributes(sqlite))]
pub fn from_row_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let (fields, has_skipped) = if let syn::Data::Struct(data) = input.data {
        let mut fields = Vec::new();
        let mut has_skipped = false;
        for field in data.fields {
            let mut skip = false;
            for attr in &field.attrs {
                if attr.path().is_ident("sqlite") && attr.parse_args::<Ident>().unwrap() == "skip" {
                    skip = true;
                    has_skipped = true;
                    break;
                }
            }
            if !skip {
                fields.push(field);
            }
        }
        (fields, has_skipped)
    } else {
        panic!("FromRow can only be used on structs");
    };

    let mut columns = "".to_string();
    for (i, field) in fields.iter().enumerate() {
        columns.push_str(&field.ident.as_ref().unwrap().to_string());
        if i < fields.len() - 1 {
            columns.push_str(", ");
        }
    }

    let mut values = "".to_string();
    for i in 0..fields.len() {
        values.push('?');
        if i < fields.len() - 1 {
            values.push_str(", ");
        }
    }

    let binds = fields.iter().enumerate().map(|(index, field)| {
        let field = field.ident.as_ref().unwrap();
        let index = index as i32;
        quote! { statement.bind_value(self.#field, #index) }
    });

    let from_rows = fields.iter().enumerate().map(|(index, field)| {
        let field = field.ident.as_ref().unwrap();
        let index = index as i32;
        quote! { #field: statement.read_value(#index).try_into().unwrap() }
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
