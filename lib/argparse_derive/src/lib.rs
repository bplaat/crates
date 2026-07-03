/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Derive macro's for the argparse crate

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Data, DeriveInput, Expr, Fields, GenericArgument, Lit, PathArguments, Type, parse_macro_input,
};

#[derive(Default)]
struct Attrs {
    name: Option<String>,
    short: Option<char>,
    long: Option<String>,
    aliases: Vec<String>,
    value: Option<String>,
    help: Option<String>,
    default: bool,
    subcommand: bool,
    positional: bool,
    command: Option<String>,
}

impl Attrs {
    fn is_empty(&self) -> bool {
        self.name.is_none()
            && self.short.is_none()
            && self.long.is_none()
            && self.aliases.is_empty()
            && self.value.is_none()
            && self.help.is_none()
            && !self.default
            && !self.subcommand
            && !self.positional
            && self.command.is_none()
    }
}

fn parse_attrs(attrs: &[syn::Attribute]) -> Attrs {
    let mut parsed = Attrs::default();
    for attr in attrs {
        if !attr.path().is_ident("arg") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("default") {
                parsed.default = true;
                return Ok(());
            }
            if meta.path.is_ident("subcommand") {
                parsed.subcommand = true;
                return Ok(());
            }
            if meta.path.is_ident("positional") {
                parsed.positional = true;
                return Ok(());
            }

            let value = meta.value()?;
            if meta.path.is_ident("name") {
                parsed.name = Some(parse_string(value.parse()?));
            } else if meta.path.is_ident("short") {
                parsed.short = Some(parse_char(value.parse()?));
            } else if meta.path.is_ident("long") {
                parsed.long = Some(parse_string(value.parse()?));
            } else if meta.path.is_ident("alias") {
                parsed.aliases.push(parse_string(value.parse()?));
            } else if meta.path.is_ident("value") {
                parsed.value = Some(parse_string(value.parse()?));
            } else if meta.path.is_ident("help") {
                parsed.help = Some(parse_string(value.parse()?));
            } else if meta.path.is_ident("command") {
                parsed.command = Some(parse_string(value.parse()?));
            } else {
                return Err(meta.error("Unsupported arg attribute"));
            }
            Ok(())
        })
        .expect("Invalid arg attribute");
    }
    parsed
}

fn parse_string(expr: Expr) -> String {
    if let Expr::Lit(expr_lit) = expr
        && let Lit::Str(lit) = expr_lit.lit
    {
        return lit.value();
    }
    panic!("Expected string literal");
}

fn parse_char(expr: Expr) -> char {
    if let Expr::Lit(expr_lit) = expr
        && let Lit::Char(lit) = expr_lit.lit
    {
        return lit.value();
    }
    panic!("Expected char literal");
}

fn kebab_case(name: &str) -> String {
    let mut output = String::new();
    for (index, ch) in name.chars().enumerate() {
        if ch == '_' {
            output.push('-');
        } else if ch.is_uppercase() {
            if index != 0 {
                output.push('-');
            }
            output.extend(ch.to_lowercase());
        } else {
            output.push(ch);
        }
    }
    output
}

fn type_ident(ty: &Type) -> Option<String> {
    if let Type::Path(type_path) = ty {
        type_path
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
    } else {
        None
    }
}

fn inner_type<'a>(ty: &'a Type, outer: &str) -> Option<&'a Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };
    let segment = type_path.path.segments.last()?;
    if segment.ident != outer {
        return None;
    }
    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };
    let Some(GenericArgument::Type(inner)) = args.args.first() else {
        return None;
    };
    Some(inner)
}

/// Parser derive
#[proc_macro_derive(Parser, attributes(arg))]
pub fn parser_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let attrs = parse_attrs(&input.attrs);
    let command_name = attrs
        .name
        .unwrap_or_else(|| std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "app".to_string()));

    let data = match input.data {
        Data::Struct(data) => data,
        _ => panic!("Parser can only be derived for structs"),
    };
    let Fields::Named(fields) = data.fields else {
        panic!("Parser can only be derived for named structs");
    };

    let mut options = Vec::new();
    let mut option_matches = Vec::new();
    let mut positional_fields = Vec::new();
    let mut subcommand_field = None;
    let mut help_field = None;
    let mut version_field = None;
    let mut usage_parts = Vec::new();

    for field in fields.named {
        let field_name = field.ident.expect("Invalid field");
        let field_name_string = field_name.to_string();
        let field_attrs = parse_attrs(&field.attrs);
        if field_attrs.subcommand {
            subcommand_field = Some((field_name, field.ty));
            usage_parts.push("[SUBCOMMAND]".to_string());
            continue;
        }

        if field_attrs.positional {
            positional_fields.push((
                field_name,
                field.ty,
                field_attrs.value.unwrap_or(field_name_string),
                field_attrs.command,
            ));
            continue;
        }

        if field_attrs.is_empty() {
            continue;
        }

        let short = field_attrs.short;
        let long = field_attrs
            .long
            .unwrap_or_else(|| kebab_case(&field_name_string));
        let help = field_attrs.help.unwrap_or_default();
        let value = field_attrs.value;
        let field_ty = field.ty;
        let is_bool = type_ident(&field_ty).as_deref() == Some("bool");
        if is_bool && field_name_string == "help" {
            help_field = Some(field_name.clone());
        }
        if is_bool && field_name_string == "version" {
            version_field = Some(field_name.clone());
        }
        let is_vec = inner_type(&field_ty, "Vec").is_some();
        let value_name = value.clone().or_else(|| {
            if is_bool {
                None
            } else {
                Some("value".to_string())
            }
        });
        let short_token = short.map_or_else(|| quote! { None }, |short| quote! { Some(#short) });
        let value_token = value_name
            .as_ref()
            .map_or_else(|| quote! { None }, |value| quote! { Some(#value) });
        options.push(quote! {
            ::argparse::OptionSpec {
                short: #short_token,
                long: #long,
                value: #value_token,
                help: #help,
            }
        });

        let mut arms = Vec::new();
        if let Some(short) = short {
            arms.push(format!("-{short}"));
        }
        arms.push(format!("--{long}"));
        for alias in &field_attrs.aliases {
            arms.push(format!("--{alias}"));
        }
        let arms = arms.iter().map(|arm| quote! { #arm });

        if is_bool {
            option_matches.push(quote! {
                #(#arms)|* => {
                    args.#field_name = true;
                }
            });
        } else if is_vec {
            let inner = inner_type(&field_ty, "Vec").expect("Invalid Vec field");
            option_matches.push(quote! {
                #(#arms)|* => {
                    let value = iter.next().ok_or_else(|| ::argparse::Error::new(format!("Expected value after {arg}")))?;
                    args.#field_name.push(value.parse::<#inner>().map_err(|_| ::argparse::Error::new(format!("Invalid value for {arg}: {value}")))?);
                }
            });
        } else if let Some(inner) = inner_type(&field_ty, "Option") {
            option_matches.push(quote! {
                #(#arms)|* => {
                    let value = iter.next().ok_or_else(|| ::argparse::Error::new(format!("Expected value after {arg}")))?;
                    args.#field_name = Some(value.parse::<#inner>().map_err(|_| ::argparse::Error::new(format!("Invalid value for {arg}: {value}")))?);
                }
            });
        } else {
            option_matches.push(quote! {
                #(#arms)|* => {
                    let value = iter.next().ok_or_else(|| ::argparse::Error::new(format!("Expected value after {arg}")))?;
                    args.#field_name = value.parse::<#field_ty>().map_err(|_| ::argparse::Error::new(format!("Invalid value for {arg}: {value}")))?;
                }
            });
        }
    }

    let usage = if usage_parts.is_empty() {
        format!("{command_name} [OPTIONS]")
    } else {
        format!("{command_name} {} [OPTIONS]", usage_parts.join(" "))
    };

    let mut positional_parsers = Vec::new();
    let mut positional_names = Vec::new();
    for (index, (field_name, field_ty, value_name, command)) in positional_fields.iter().enumerate()
    {
        if command.is_none() {
            positional_names.push(value_name.clone());
        }
        let index = syn::Index::from(index);
        let command_check = if let Some(command) = command {
            let Some((subcommand_field_name, subcommand_field_ty)) = &subcommand_field else {
                panic!("Command-scoped positional requires a subcommand field");
            };
            quote! {
                if <#subcommand_field_ty as ::argparse::Subcommand>::parse_subcommand(#command)
                    .is_some_and(|command| args.#subcommand_field_name == command)
                {
                } else {
                    return Err(::argparse::Error::new(format!("Unknown argument: {arg}")));
                }
            }
        } else {
            quote! {}
        };
        if inner_type(field_ty, "Vec").is_some() {
            let inner = inner_type(field_ty, "Vec").expect("Invalid Vec field");
            positional_parsers.push(quote! {
                #index => {
                    #command_check
                    args.#field_name.push(arg.parse::<#inner>().map_err(|_| ::argparse::Error::new(format!("Invalid value: {arg}")))?);
                }
            });
        } else if let Some(inner) = inner_type(field_ty, "Option") {
            positional_parsers.push(quote! {
                #index => {
                    #command_check
                    args.#field_name = Some(arg.parse::<#inner>().map_err(|_| ::argparse::Error::new(format!("Invalid value: {arg}")))?);
                    positional_index += 1;
                }
            });
        } else {
            positional_parsers.push(quote! {
                #index => {
                    #command_check
                    args.#field_name = arg.parse::<#field_ty>().map_err(|_| ::argparse::Error::new(format!("Invalid value: {arg}")))?;
                    positional_index += 1;
                }
            });
        }
    }

    let positional_usage = if positional_names.is_empty() {
        usage
    } else {
        format!(
            "{} {}",
            usage.replace(" [OPTIONS]", ""),
            positional_names
                .iter()
                .map(|name| format!("<{name}>"))
                .collect::<Vec<_>>()
                .join(" ")
        ) + " [OPTIONS]"
    };

    let has_help_field = help_field.is_some();
    let has_version_field = version_field.is_some();
    let (subcommand_parse, subcommands_expr, help_command, version_command) = if let Some((
        field_name,
        field_ty,
    )) =
        subcommand_field
    {
        (
            quote! {
                if !arg.starts_with('-') {
                    if let Some(command) = <#field_ty as ::argparse::Subcommand>::parse_subcommand(&arg) {
                        args.#field_name = command;
                        continue;
                    }
                }
            },
            quote! { <#field_ty as ::argparse::Subcommand>::subcommands() },
            quote! {
                if arg == "-h" || arg == "--help" || arg == "help" {
                    if let Some(command) = <#field_ty as ::argparse::Subcommand>::help_command() {
                        args.#field_name = command;
                        continue;
                    }
                }
            },
            quote! {
                if arg == "-v" || arg == "--version" || arg == "version" {
                    if let Some(command) = <#field_ty as ::argparse::Subcommand>::version_command() {
                        args.#field_name = command;
                        continue;
                    }
                }
            },
        )
    } else {
        let help_command = if let Some(field_name) = help_field {
            quote! {
                if matches!(arg.as_str(), "-h" | "--help" | "help") {
                    args.#field_name = true;
                    continue;
                }
            }
        } else {
            quote! {
                if matches!(arg.as_str(), "-h" | "--help" | "help") {
                    return Err(::argparse::Error::help(Self::help()));
                }
            }
        };
        let version_command = if let Some(field_name) = version_field {
            quote! {
                if matches!(arg.as_str(), "-v" | "--version" | "version") {
                    args.#field_name = true;
                    continue;
                }
            }
        } else {
            quote! {
                if matches!(arg.as_str(), "-v" | "--version" | "version") {
                    return Err(::argparse::Error::version(Self::version()));
                }
            }
        };
        (quote! {}, quote! { &[] }, help_command, version_command)
    };

    let positional_match = if positional_parsers.is_empty() {
        quote! {
            return Err(::argparse::Error::new(format!("Unknown argument: {arg}")));
        }
    } else {
        quote! {
            match positional_index {
                #(#positional_parsers,)*
                _ => return Err(::argparse::Error::new(format!("Unknown argument: {arg}"))),
            }
        }
    };

    TokenStream::from(quote! {
        impl #name {
            pub fn parse() -> Self {
                <Self as ::argparse::Parser>::parse()
            }

            pub fn parse_from<I, S>(args: I) -> ::argparse::Result<Self>
            where
                I: IntoIterator<Item = S>,
                S: Into<String>,
            {
                <Self as ::argparse::Parser>::parse_from(args)
            }

            pub fn help() -> String {
                <Self as ::argparse::Parser>::help()
            }

            pub fn version() -> String {
                <Self as ::argparse::Parser>::version()
            }
        }

        impl ::argparse::Parser for #name {
            fn parse_from<I, S>(raw_args: I) -> ::argparse::Result<Self>
            where
                I: IntoIterator<Item = S>,
                S: Into<String>,
            {
                let mut args = Self::default();
                let mut iter = raw_args.into_iter().map(Into::into).peekable();
                let mut positional_index = 0usize;
                while let Some(arg) = iter.next() {
                    #help_command
                    #version_command
                    match arg.as_str() {
                        #(#option_matches,)*
                        _ => {
                            #subcommand_parse
                            if arg.starts_with('-') {
                                return Err(::argparse::Error::new(format!("Unknown argument: {arg}")));
                            }
                            #positional_match
                        }
                    }
                }
                Ok(args)
            }

            fn help() -> String {
                ::argparse::format_help(
                    #positional_usage,
                    &[#(#options),*],
                    #subcommands_expr,
                    !#has_help_field,
                    !#has_version_field,
                )
            }

            fn version() -> String {
                format!("{} v{}", #command_name, env!("CARGO_PKG_VERSION"))
            }
        }
    })
}

/// Subcommand derive
#[proc_macro_derive(Subcommand, attributes(arg))]
pub fn subcommand_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let data = match input.data {
        Data::Enum(data) => data,
        _ => panic!("Subcommand can only be derived for enums"),
    };

    let mut parse_arms = Vec::new();
    let mut commands = Vec::new();
    let mut help_command = quote! { None };
    let mut version_command = quote! { None };

    for variant in data.variants {
        let variant_name = variant.ident;
        let attrs = parse_attrs(&variant.attrs);
        let command_name = attrs
            .name
            .unwrap_or_else(|| kebab_case(&variant_name.to_string()));
        let aliases = attrs.aliases;
        let help = attrs.help.unwrap_or_default();
        let mut names = vec![command_name.clone()];
        names.extend(aliases.iter().cloned());
        let names = names.iter().map(|name| quote! { #name });
        parse_arms.push(quote! {
            #(#names)|* => Some(Self::#variant_name),
        });
        let aliases_tokens = aliases.iter().map(|alias| quote! { #alias });
        commands.push(quote! {
            ::argparse::Command {
                name: #command_name,
                aliases: &[#(#aliases_tokens),*],
                help: #help,
            }
        });

        let lower = variant_name.to_string().to_lowercase();
        if lower == "help" {
            help_command = quote! { Some(Self::#variant_name) };
        }
        if lower == "version" {
            version_command = quote! { Some(Self::#variant_name) };
        }
    }

    let commands_ident = format_ident!(
        "__ARGPARSER_COMMANDS_FOR_{}",
        name.to_string().to_uppercase()
    );
    TokenStream::from(quote! {
        static #commands_ident: &[::argparse::Command] = &[#(#commands),*];

        impl ::argparse::Subcommand for #name {
            fn parse_subcommand(command: &str) -> Option<Self> {
                match command {
                    #(#parse_arms)*
                    _ => None,
                }
            }

            fn subcommands() -> &'static [::argparse::Command] {
                #commands_ident
            }

            fn help_command() -> Option<Self> {
                #help_command
            }

            fn version_command() -> Option<Self> {
                #version_command
            }
        }
    })
}
