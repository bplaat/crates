/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A [FromEnum] and [FromStruct] derive macro library

use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, DeriveInput, Meta, Path, Type, parse_macro_input};

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

    let (other_name, from, into) = conversion_config(&input.attrs, "from_enum");

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
    let into_impl = into.then(|| {
        quote! {
            impl #impl_generics From<#name #type_generics> for #other_name #type_generics #where_clause {
                fn from(value: #name #type_generics) -> Self {
                    match value {
                        #(#variants)*
                    }
                }
            }
        }
    });
    let from_impl = from.then(|| {
        quote! {
            impl #impl_generics From<#other_name #type_generics> for #name #type_generics #where_clause {
                fn from(value: #other_name #type_generics) -> Self {
                    match value {
                        #(#variants_reverse)*
                    }
                }
            }
        }
    });
    TokenStream::from(quote! {
        #into_impl
        #from_impl
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

    let (other_name, from, into) = conversion_config(&input.attrs, "from_struct");

    // Generate code
    let fields_into = data.fields.iter().map(|field| {
        let field_name = &field.ident;
        quote! {
            #field_name: value.#field_name.into(),
        }
    });
    let fields_from = data.fields.iter().map(|field| {
        let field_name = &field.ident;
        if is_option(&field.ty) {
            quote! {
                #field_name: value.#field_name.map(Into::into),
            }
        } else {
            quote! {
                #field_name: value.#field_name.into(),
            }
        }
    });
    let into_impl = into.then(|| {
        quote! {
            impl #impl_generics From<#name #type_generics> for #other_name #type_generics #where_clause {
                fn from(value: #name #type_generics) -> Self {
                    #other_name {
                        #(#fields_into)*
                    }
                }
            }
        }
    });
    let from_impl = from.then(|| {
        quote! {
            impl #impl_generics From<#other_name #type_generics> for #name #type_generics #where_clause {
                fn from(value: #other_name #type_generics) -> Self {
                    #name {
                        #(#fields_from)*
                    }
                }
            }
        }
    });
    TokenStream::from(quote! {
        #into_impl
        #from_impl
    })
}

fn conversion_config(attributes: &[Attribute], attribute_name: &str) -> (Path, bool, bool) {
    let mut other_name = None;
    let mut only_from = false;
    let mut only_into = false;
    for attribute in attributes {
        if attribute.path().is_ident(attribute_name) {
            let list = attribute
                .parse_args_with(
                    syn::punctuated::Punctuated::<_, syn::token::Comma>::parse_terminated,
                )
                .expect("Invalid attribute");
            for item in list {
                if let Meta::Path(path) = item {
                    if path.is_ident("only_from") {
                        only_from = true;
                    } else if path.is_ident("only_into") {
                        only_into = true;
                    } else {
                        other_name = Some(path);
                    }
                }
            }
        }
    }
    let other_name = other_name.unwrap_or_else(|| panic!("Missing {attribute_name} attribute"));
    if only_from && only_into {
        panic!("only_from and only_into cannot be combined");
    }
    if only_from {
        (other_name, true, false)
    } else if only_into {
        (other_name, false, true)
    } else {
        (other_name, true, true)
    }
}

fn is_option(target: &Type) -> bool {
    matches!(target, Type::Path(path) if path.path.segments.last().is_some_and(|segment| segment.ident == "Option"))
}
