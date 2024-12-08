/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

extern crate proc_macro;
use std::fmt::Display;
use std::str::FromStr;

use proc_macro::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, DeriveInput, Expr, ExprLit, Ident, Lit, Meta};

enum Rule {
    Ascii,
    #[cfg(feature = "email")]
    Email,
    LengthMin(usize),
    LengthMax(usize),
    RangeMin(i64),
    RangeMax(i64),
    Custom(Ident),
}

#[proc_macro_derive(Validate, attributes(validate))]
pub fn validate_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    // Parse context type
    let mut context = None;
    for attr in input.attrs {
        if attr.path().is_ident("validate") {
            let list = attr
                .parse_args_with(Punctuated::<_, syn::token::Comma>::parse_terminated)
                .unwrap();
            for item in list {
                if let Meta::List(meta_list) = item {
                    if meta_list.path.is_ident("context") {
                        let list = meta_list
                            .parse_args_with(Punctuated::<_, syn::token::Comma>::parse_terminated)
                            .unwrap();
                        if let Meta::Path(path) = &list[0] {
                            context = Some(path.get_ident().unwrap().clone());
                        }
                    }
                }
            }
        }
    }

    // Parse fields with #[validate] attribute
    let fields = if let syn::Data::Struct(data) = input.data {
        let mut fields = Vec::new();
        for field in data.fields {
            let mut rules = Vec::new();
            for attr in field.attrs.iter() {
                if attr.path().is_ident("validate") {
                    let list = attr
                        .parse_args_with(Punctuated::<_, syn::token::Comma>::parse_terminated)
                        .unwrap();
                    for item in list {
                        match item {
                            Meta::Path(path) => {
                                if path.is_ident("ascii") {
                                    rules.push(Rule::Ascii);
                                }
                                #[cfg(feature = "email")]
                                if path.is_ident("email") {
                                    rules.push(Rule::Email);
                                }
                            }
                            Meta::List(meta_list) => {
                                let list = meta_list
                                    .parse_args_with(
                                        Punctuated::<_, syn::token::Comma>::parse_terminated,
                                    )
                                    .unwrap();
                                if meta_list.path.is_ident("length") {
                                    for item in &list {
                                        if let Meta::NameValue(name_value) = item {
                                            if name_value.path.is_ident("min") {
                                                rules.push(Rule::LengthMin(
                                                    expr_to::<usize>(&name_value.value).unwrap(),
                                                ));
                                            }
                                            if name_value.path.is_ident("max") {
                                                rules.push(Rule::LengthMax(
                                                    expr_to::<usize>(&name_value.value).unwrap(),
                                                ));
                                            }
                                        }
                                    }
                                }
                                if meta_list.path.is_ident("range") {
                                    for item in &list {
                                        if let Meta::NameValue(name_value) = item {
                                            if name_value.path.is_ident("min") {
                                                rules.push(Rule::RangeMin(
                                                    expr_to::<i64>(&name_value.value).unwrap(),
                                                ));
                                            }
                                            if name_value.path.is_ident("max") {
                                                rules.push(Rule::RangeMax(
                                                    expr_to::<i64>(&name_value.value).unwrap(),
                                                ));
                                            }
                                        }
                                    }
                                }
                                if meta_list.path.is_ident("custom") {
                                    for item in &list {
                                        if let Meta::Path(path) = item {
                                            rules.push(Rule::Custom(
                                                path.get_ident().unwrap().clone(),
                                            ));
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            fields.push((field, rules));
        }
        fields
    } else {
        panic!("Validate can only be used on structs");
    };

    // Generate code
    let context_type = match &context {
        Some(context) => quote! { #context },
        None => quote! { () },
    };

    let validate_fields = fields.iter().map(|(field, rules)| {
        let field_name = field.ident.as_ref().unwrap();
        let validate_rules = rules.iter().map(|rule| match rule {
            Rule::Ascii => quote! {
                if !self.#field_name.is_ascii() {
                    errors
                        .entry(stringify!(#field_name).to_string())
                        .or_insert_with(Vec::new)
                        .push(format!("{} must only contain ascii characters", stringify!(#field_name)));
                }
            },
            #[cfg(feature = "email")]
            Rule::Email => quote! {
                if !validate::is_valid_email(&self.#field_name) {
                    errors
                        .entry(stringify!(#field_name).to_string())
                        .or_insert_with(Vec::new)
                        .push(format!("{} must be a valid email address", stringify!(#field_name)));
                }
            },
            Rule::LengthMin(min) => quote! {
                if self.#field_name.len() < #min {
                    errors
                        .entry(stringify!(#field_name).to_string())
                        .or_insert_with(Vec::new)
                        .push(format!("{} must be at least {} characters long", stringify!(#field_name), #min));
                }
            },
            Rule::LengthMax(max) => quote! {
                if self.#field_name.len() > #max {
                    errors
                        .entry(stringify!(#field_name).to_string())
                        .or_insert_with(Vec::new)
                        .push(format!("{} must be at most {} characters long", stringify!(#field_name), #max));
                }
            },
            Rule::RangeMin(min) => quote! {
                if self.#field_name < #min {
                    errors
                        .entry(stringify!(#field_name).to_string())
                        .or_insert_with(Vec::new)
                        .push(format!("{} must be at least {}", stringify!(#field_name), #min));
                }
            },
            Rule::RangeMax(max) => quote! {
                if self.#field_name > #max {
                    errors
                        .entry(stringify!(#field_name).to_string())
                        .or_insert_with(Vec::new)
                        .push(format!("{} must be at most {}", stringify!(#field_name), #max));
                }
            },
            Rule::Custom(custom) => if context.is_some() {
                quote! {
                    if let Err(error) = #custom(&self.#field_name, context) {
                        errors
                            .entry(stringify!(#field_name).to_string())
                            .or_insert_with(Vec::new)
                            .push(error.message().to_string());
                    }
                }
            } else {
                quote! {
                    if let Err(error) = #custom(&self.#field_name) {
                        errors
                            .entry(stringify!(#field_name).to_string())
                            .or_insert_with(Vec::new)
                            .push(error.message().to_string());
                    }
                }
            },
        });
        quote! {
            #(#validate_rules)*
        }
    });

    TokenStream::from(quote! {
        impl validate::Validate for #name {
            type Context = #context_type;
            fn validate_with(&self, context: &Self::Context) -> std::result::Result<(), validate::Errors> {
                let mut errors = std::collections::BTreeMap::new();
                #(#validate_fields;)*
                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(validate::Errors(errors))
                }
            }
        }
    })
}

fn expr_to<N>(expr: &Expr) -> Option<N>
where
    N: FromStr,
    N::Err: Display,
{
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Int(lit_int),
            ..
        }) => lit_int.base10_parse::<N>().ok(),
        _ => None,
    }
}
