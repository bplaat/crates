/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Validation derive macro's library

#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{DeriveInput, Expr, Meta, parse_macro_input};

struct Rule {
    r#type: RuleType,
    is_option: bool,
}

enum RuleType {
    Ascii,
    #[cfg(feature = "email")]
    Email,
    #[cfg(feature = "url")]
    Url,
    LengthMin(Expr),
    LengthMax(Expr),
    RangeMin(Expr),
    RangeMax(Expr),
    Custom(Expr),
}

/// [Validate] derive
#[proc_macro_derive(Validate, attributes(validate))]
pub fn validate_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    // Parse context type
    let mut context = None;
    for attr in input.attrs {
        if attr.path().is_ident("validate") {
            let list = attr
                .parse_args_with(
                    syn::punctuated::Punctuated::<_, syn::token::Comma>::parse_terminated,
                )
                .expect("Invalid attribute");
            for item in list {
                if let Meta::List(meta_list) = item
                    && meta_list.path.is_ident("context")
                {
                    let list = meta_list
                        .parse_args_with(
                            syn::punctuated::Punctuated::<_, syn::token::Comma>::parse_terminated,
                        )
                        .expect("Invalid attribute");
                    if let Meta::Path(path) = &list[0] {
                        context = Some(path.get_ident().expect("Invalid attribute").clone());
                    }
                }
            }
        }
    }

    // Parse fields with #[validate] attribute
    let fields = if let syn::Data::Struct(data) = input.data {
        let mut fields = Vec::new();
        for field in data.fields {
            let is_option = field.ty.to_token_stream().to_string().starts_with("Option");
            let mut rules = Vec::new();
            for attr in field.attrs.iter() {
                if attr.path().is_ident("validate") {
                    let list = attr
                        .parse_args_with(
                            syn::punctuated::Punctuated::<_, syn::token::Comma>::parse_terminated,
                        )
                        .expect("Invalid attribute");
                    for item in list {
                        match item {
                            Meta::Path(path) => {
                                if path.is_ident("ascii") {
                                    rules.push(Rule {
                                        r#type: RuleType::Ascii,
                                        is_option,
                                    });
                                }
                                #[cfg(feature = "email")]
                                if path.is_ident("email") {
                                    rules.push(Rule {
                                        r#type: RuleType::Email,
                                        is_option,
                                    });
                                }
                                #[cfg(feature = "url")]
                                if path.is_ident("url") {
                                    rules.push(Rule {
                                        r#type: RuleType::Url,
                                        is_option,
                                    });
                                }
                            }
                            Meta::List(meta_list) => {
                                let list = meta_list
                                    .parse_args_with(
                                        syn::punctuated::Punctuated::<_, syn::token::Comma>::parse_terminated,
                                    )
                                    .expect("Invalid attribute");
                                if meta_list.path.is_ident("length") {
                                    for item in &list {
                                        if let Meta::NameValue(name_value) = item {
                                            if name_value.path.is_ident("min") {
                                                rules.push(Rule {
                                                    r#type: RuleType::LengthMin(
                                                        name_value.value.clone(),
                                                    ),
                                                    is_option,
                                                });
                                            }
                                            if name_value.path.is_ident("max") {
                                                rules.push(Rule {
                                                    r#type: RuleType::LengthMax(
                                                        name_value.value.clone(),
                                                    ),
                                                    is_option,
                                                });
                                            }
                                        }
                                    }
                                }
                                if meta_list.path.is_ident("range") {
                                    for item in &list {
                                        if let Meta::NameValue(name_value) = item {
                                            if name_value.path.is_ident("min") {
                                                rules.push(Rule {
                                                    r#type: RuleType::RangeMin(
                                                        name_value.value.clone(),
                                                    ),
                                                    is_option,
                                                });
                                            }
                                            if name_value.path.is_ident("max") {
                                                rules.push(Rule {
                                                    r#type: RuleType::RangeMax(
                                                        name_value.value.clone(),
                                                    ),
                                                    is_option,
                                                });
                                            }
                                        }
                                    }
                                }
                                if meta_list.path.is_ident("custom") {
                                    for item in &list {
                                        if let Meta::Path(path) = item {
                                            rules.push(Rule {
                                                r#type: RuleType::Custom(
                                                    syn::parse2(path.to_token_stream())
                                                        .expect("Invalid attribute"),
                                                ),
                                                is_option,
                                            });
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
        let field_name = field.ident.as_ref().expect("Invalid field");
        let validate_rules = rules.iter().map(|rule| {
            let test_condition = |condition, error| {
                let field_name_string = field_name.to_string().replace("r#", "");
                if rule.is_option {
                    quote! {
                        if let Some(value) = &self.#field_name {
                            if #condition {
                                report.insert_error(#field_name_string, #error);
                            }
                        }
                    }
                } else {
                    quote! {
                        let value = &self.#field_name;
                        if #condition {
                            report.insert_error(#field_name_string, #error);
                        }
                    }
                }
            };

            match &rule.r#type {
                RuleType::Ascii => test_condition(
                    quote! { !value.is_ascii() },
                    quote! { "must only contain ASCII characters".to_string() },
                ),
                #[cfg(feature = "email")]
                RuleType::Email => test_condition(
                    quote! { !validate::is_valid_email(value) },
                    quote! { "must be a valid email address".to_string() },
                ),
                #[cfg(feature = "url")]
                RuleType::Url => test_condition(
                    quote! { !validate::is_valid_url(value) },
                    quote! { "must be a valid url".to_string() },
                ),
                RuleType::LengthMin(min) => test_condition(
                    quote! { value.len() < #min as usize },
                    quote! { format!("must be at least {} characters long", #min) },
                ),
                RuleType::LengthMax(max) => test_condition(
                    quote! { value.len() > #max as usize },
                    quote! { format!("must be at most {} characters long", #max) },
                ),
                RuleType::RangeMin(min) => test_condition(
                    quote! { *value < #min },
                    quote! { format!("must be at least {}", #min) },
                ),
                RuleType::RangeMax(max) => test_condition(
                    quote! { *value > #max },
                    quote! { format!("must be at most {}", #max) },
                ),
                RuleType::Custom(custom) => {
                    if context.is_some() {
                        if rule.is_option {
                            quote! {
                                if let Some(value) = &self.#field_name {
                                    if let Err(err) = #custom(value, context) {
                                        report.insert_error(stringify!(#field_name), err.message());
                                    }
                                }
                            }
                        } else {
                            quote! {
                                let value = &self.#field_name;
                                if let Err(err) = #custom(value, context) {
                                    report.insert_error(stringify!(#field_name), err.message());
                                }
                            }
                        }
                    } else if rule.is_option {
                        quote! {
                            if let Some(value) = &self.#field_name {
                                if let Err(err) = #custom(value) {
                                    report.insert_error(stringify!(#field_name), err.message());
                                }
                            }
                        }
                    } else {
                        quote! {
                            let value = &self.#field_name;
                            if let Err(err) = #custom(value) {
                                report.insert_error(stringify!(#field_name), err.message());
                            }
                        }
                    }
                }
            }
        });
        quote! {
            #(#validate_rules)*
        }
    });

    TokenStream::from(quote! {
        impl validate::Validate for #name {
            type Context = #context_type;
            fn validate_with(&self, context: &Self::Context) -> std::result::Result<(), validate::Report> {
                let mut report = validate::Report::new();
                #(#validate_fields;)*
                if report.is_empty() {
                    Ok(())
                } else {
                    Err(report)
                }
            }
        }
    })
}
