/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(FromRow, attributes(sqlite))]
pub fn from_row_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let fields = if let syn::Data::Struct(data) = input.data {
        data.fields
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

    let mut sets = "".to_string();
    for (i, field) in fields.iter().enumerate() {
        sets.push_str(&format!("{} = ?", field.ident.as_ref().unwrap()));
        if i < fields.len() - 1 {
            sets.push_str(", ");
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
        let index = index as i32 + 1;
        quote! {
            let value: sqlite::Value = self.#field.into();
            value.bind_to_statement(statement, #index)?;
        }
    });

    let from_rows = fields.iter().enumerate().map(|(index, field)| {
        let field = field.ident.as_ref().unwrap();
        let index = index as i32;
        quote! { #field: sqlite::Value::read_from_statement(statement, #index)?.try_into()? }
    });

    TokenStream::from(quote! {
        impl #name {
            pub fn columns() -> &'static str {
                #columns
            }
            pub fn values() -> &'static str {
                #values
            }
            pub fn sets() -> &'static str {
                #sets
            }
        }
        impl sqlite::Bind for #name {
            fn bind(self, statement: *mut sqlite::sys::sqlite3_stmt) -> sqlite::Result<()> {
                #( #binds )*
                Ok(())
            }
        }
        impl sqlite::FromRow for #name {
            fn from_row(statement: *mut sqlite::sys::sqlite3_stmt) -> sqlite::Result<Self> {
                Ok(Self {
                    #( #from_rows, )*
                })
            }
        }
    })
}
