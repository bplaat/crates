/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Minimal clap-compatible derive macro library

#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{DeriveInput, Expr, Lit, Meta, parse_macro_input, punctuated::Punctuated, token::Comma};

// MARK: Helpers

fn camel_to_kebab(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            out.push('-');
        }
        out.extend(c.to_lowercase());
    }
    out
}

fn snake_to_kebab(s: &str) -> String {
    s.replace('_', "-")
}

// Build `a || b || c` expression from a list of conditions
fn or_conditions(conds: &[String]) -> TokenStream2 {
    conds
        .iter()
        .map(|c| quote! { _arg == #c })
        .reduce(|a, b| quote! { #a || #b })
        .unwrap_or_else(|| quote! { false })
}

#[derive(PartialEq, Eq)]
enum TypeKind {
    Bool,
    Str,
    Other,
    OptionStr,
    OptionOther,
    VecStr,
    VecOther,
}

fn classify(ty: &syn::Type) -> TypeKind {
    let ts = quote!(#ty).to_string().replace(' ', "");
    match ts.as_str() {
        "bool" => TypeKind::Bool,
        "String" => TypeKind::Str,
        s if s.starts_with("Option<") => {
            if &s[7..s.len() - 1] == "String" {
                TypeKind::OptionStr
            } else {
                TypeKind::OptionOther
            }
        }
        s if s.starts_with("Vec<") => {
            if &s[4..s.len() - 1] == "String" {
                TypeKind::VecStr
            } else {
                TypeKind::VecOther
            }
        }
        _ => TypeKind::Other,
    }
}

fn inner_ty(ty: &syn::Type) -> TokenStream2 {
    if let syn::Type::Path(tp) = ty
        && let Some(seg) = tp.path.segments.last()
        && let syn::PathArguments::AngleBracketed(ab) = &seg.arguments
        && let Some(syn::GenericArgument::Type(inner)) = ab.args.first()
    {
        return quote!(#inner);
    }
    quote!(#ty)
}

fn str_from_meta(meta: &Meta) -> Option<String> {
    if let Meta::NameValue(nv) = meta
        && let Expr::Lit(el) = &nv.value
        && let Lit::Str(s) = &el.lit
    {
        return Some(s.value());
    }
    None
}

// MARK: Field config

struct FieldConfig {
    ident: syn::Ident,
    ty: syn::Type,
    short: Option<char>,
    long: Option<String>,
    aliases: Vec<String>,
    help: Option<String>,
    default_value: Option<String>,
    value_name: Option<String>,
    is_subcommand: bool,
    cfg_attrs: Vec<syn::Attribute>,
}

impl FieldConfig {
    fn from_field(field: &syn::Field) -> Self {
        let ident = field.ident.clone().expect("Unnamed fields not supported");
        let field_name = ident.to_string();

        let mut short = None;
        let mut long = None;
        let mut aliases = Vec::new();
        let mut help = None;
        let mut default_value = None;
        let mut value_name = None;
        let mut is_subcommand = false;

        let cfg_attrs: Vec<syn::Attribute> = field
            .attrs
            .iter()
            .filter(|a| a.path().is_ident("cfg"))
            .cloned()
            .collect();

        for attr in &field.attrs {
            if attr.path().is_ident("arg") || attr.path().is_ident("command") {
                let items = attr
                    .parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
                    .expect("Invalid attribute");
                for item in &items {
                    match item {
                        Meta::Path(p) if p.is_ident("short") => {
                            short = Some(field_name.chars().next().unwrap_or('x'));
                        }
                        Meta::Path(p) if p.is_ident("long") => {
                            long = Some(snake_to_kebab(&field_name));
                        }
                        Meta::Path(p) if p.is_ident("subcommand") => {
                            is_subcommand = true;
                        }
                        Meta::NameValue(nv) if nv.path.is_ident("short") => {
                            if let Expr::Lit(el) = &nv.value
                                && let Lit::Char(c) = &el.lit
                            {
                                short = Some(c.value());
                            }
                        }
                        Meta::NameValue(nv) if nv.path.is_ident("long") => {
                            long = str_from_meta(item);
                        }
                        Meta::NameValue(nv) if nv.path.is_ident("alias") => {
                            if let Some(s) = str_from_meta(item) {
                                aliases.push(s);
                            }
                        }
                        Meta::NameValue(nv) if nv.path.is_ident("help") => {
                            help = str_from_meta(item);
                        }
                        Meta::NameValue(nv) if nv.path.is_ident("default_value") => {
                            default_value = str_from_meta(item);
                        }
                        Meta::NameValue(nv) if nv.path.is_ident("value_name") => {
                            value_name = str_from_meta(item);
                        }
                        _ => {}
                    }
                }
            }
        }

        FieldConfig {
            ident,
            ty: field.ty.clone(),
            short,
            long,
            aliases,
            help,
            default_value,
            value_name,
            is_subcommand,
            cfg_attrs,
        }
    }

    fn is_named(&self) -> bool {
        !self.is_subcommand && (self.short.is_some() || self.long.is_some())
    }

    fn is_positional(&self) -> bool {
        !self.is_subcommand && self.short.is_none() && self.long.is_none()
    }

    // Build flag match pattern string like "-v" || "--verbose"
    fn match_conditions(&self) -> Vec<String> {
        let mut conds = Vec::new();
        if let Some(c) = self.short {
            conds.push(format!("-{c}"));
        }
        if let Some(ref l) = self.long {
            conds.push(format!("--{l}"));
        }
        for a in &self.aliases {
            conds.push(format!("--{a}"));
        }
        conds
    }

    // Help display string: "  -v, --verbose" or "  -I, --include <PATH>"
    fn help_display(&self) -> String {
        let mut parts = Vec::new();
        if let Some(c) = self.short {
            parts.push(format!("-{c}"));
        }
        if let Some(ref l) = self.long {
            parts.push(format!("--{l}"));
        }
        let flag = parts.join(", ");
        let kind = classify(&self.ty);
        if kind == TypeKind::Bool {
            format!("  {flag}")
        } else {
            let vn = self
                .value_name
                .as_deref()
                .unwrap_or("VALUE")
                .to_uppercase();
            format!("  {flag} <{vn}>")
        }
    }
}

// MARK: Struct command config

struct CommandConfig {
    about: Option<String>,
    version: bool,
}

impl CommandConfig {
    fn from_attrs(attrs: &[syn::Attribute]) -> Self {
        let mut about = None;
        let mut version = false;
        for attr in attrs {
            if attr.path().is_ident("command") {
                let items = attr
                    .parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
                    .expect("Invalid #[command] attribute");
                for item in &items {
                    match item {
                        Meta::NameValue(nv) if nv.path.is_ident("about") => {
                            about = str_from_meta(item);
                        }
                        Meta::Path(p) if p.is_ident("version") => {
                            version = true;
                        }
                        _ => {}
                    }
                }
            }
        }
        CommandConfig { about, version }
    }
}

// MARK: Help text generation

fn gen_help_print(cmd: &CommandConfig, fields: &[FieldConfig]) -> TokenStream2 {
    let mut stmts = Vec::new();

    // About
    if let Some(ref about) = cmd.about {
        stmts.push(quote! { print!("{}\n\n", #about); });
    }

    // Usage line
    let mut usage_parts = vec!["[OPTIONS]".to_string()];
    let has_subcommand = fields.iter().any(|f| f.is_subcommand);
    if has_subcommand {
        usage_parts.push("[SUBCOMMAND]".to_string());
    }
    for f in fields.iter().filter(|f| f.is_positional()) {
        let kind = classify(&f.ty);
        let ident_upper = f.ident.to_string().to_uppercase();
        let vn = f.value_name.as_deref().unwrap_or(&ident_upper);
        let s = if matches!(kind, TypeKind::VecStr | TypeKind::VecOther) {
            format!("[{vn}...]")
        } else {
            format!("[{vn}]")
        };
        usage_parts.push(s);
    }
    let usage = format!("Usage: {}", usage_parts.join(" "));
    stmts.push(quote! { print!("{}\n\nOptions:\n", #usage); });

    // Named fields (unconditional)
    let named_unconditional: Vec<_> = fields
        .iter()
        .filter(|f| f.is_named() && f.cfg_attrs.is_empty())
        .collect();
    let max_w = named_unconditional
        .iter()
        .map(|f| f.help_display().len())
        .max()
        .unwrap_or(20)
        .max(14); // at least "--help" width
    for f in &named_unconditional {
        let flag_str = f.help_display();
        let pad = max_w - flag_str.len() + 4;
        let spaces = " ".repeat(pad);
        let help = f.help.as_deref().unwrap_or("");
        let line = format!("{flag_str}{spaces}{help}\n");
        stmts.push(quote! { print!("{}", #line); });
    }

    // cfg-conditional fields (each wrapped in #[cfg])
    for f in fields.iter().filter(|f| f.is_named() && !f.cfg_attrs.is_empty()) {
        let flag_str = f.help_display();
        let pad = max_w.saturating_sub(flag_str.len()) + 4;
        let spaces = " ".repeat(pad);
        let help = f.help.as_deref().unwrap_or("");
        let line = format!("{flag_str}{spaces}{help}\n");
        let cfg = &f.cfg_attrs;
        stmts.push(quote! {
            #(#cfg)*
            print!("{}", #line);
        });
    }

    // --help and optionally --version
    let help_flag = format!("  -h, --help{}", " ".repeat(max_w - 10 + 4));
    let help_line = format!("{help_flag}Print help\n");
    stmts.push(quote! { print!("{}", #help_line); });

    if cmd.version {
        let ver_flag = format!("  -V, --version{}", " ".repeat(max_w - 13 + 4));
        let ver_line = format!("{ver_flag}Print version\n");
        stmts.push(quote! { print!("{}", #ver_line); });
    }

    quote! { #(#stmts)* }
}

// MARK: Parser derive

/// `#[derive(Parser)]` — generates `parse_from`, `parse`, and `cargo_parse` for argument structs.
#[proc_macro_derive(Parser, attributes(command, arg))]
pub fn parser_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let cmd = CommandConfig::from_attrs(&input.attrs);

    let fields_raw: Vec<&syn::Field> = match &input.data {
        syn::Data::Struct(data) => data.fields.iter().collect(),
        _ => panic!("Parser can only be derived for structs"),
    };

    let configs: Vec<FieldConfig> = fields_raw.iter().map(|f| FieldConfig::from_field(f)).collect();

    let help_print = gen_help_print(&cmd, &configs);

    // --- Declarations ---
    let decls: Vec<TokenStream2> = configs
        .iter()
        .map(|f| {
            let ident = &f.ident;
            let ty = &f.ty;
            let kind = classify(ty);
            let cfg = &f.cfg_attrs;

            if f.is_subcommand {
                let inner = inner_ty(ty);
                // Check if field type is already Option<...>
                if matches!(kind, TypeKind::OptionOther | TypeKind::OptionStr) {
                    quote! { #(#cfg)* let mut #ident: #ty = None; }
                } else {
                    quote! { #(#cfg)* let mut #ident: ::std::option::Option<#inner> = None; }
                }
            } else if f.is_positional() {
                match kind {
                    TypeKind::VecStr | TypeKind::VecOther => {
                        quote! { #(#cfg)* let mut #ident: #ty = ::std::vec::Vec::new(); }
                    }
                    TypeKind::OptionStr | TypeKind::OptionOther => {
                        quote! { #(#cfg)* let mut #ident: #ty = None; }
                    }
                    TypeKind::Bool => {
                        quote! { #(#cfg)* let mut #ident: bool = false; }
                    }
                    _ => {
                        // Positional String or T - use Option internally
                        let inner = if kind == TypeKind::Str {
                            quote!(::std::string::String)
                        } else {
                            quote!(#ty)
                        };
                        quote! { #(#cfg)* let mut #ident: ::std::option::Option<#inner> = None; }
                    }
                }
            } else {
                // Named field
                match kind {
                    TypeKind::Bool => quote! { #(#cfg)* let mut #ident: bool = false; },
                    TypeKind::Str => {
                        quote! { #(#cfg)* let mut #ident: ::std::option::Option<::std::string::String> = None; }
                    }
                    TypeKind::OptionStr | TypeKind::OptionOther => {
                        quote! { #(#cfg)* let mut #ident: #ty = None; }
                    }
                    TypeKind::VecStr | TypeKind::VecOther => {
                        quote! { #(#cfg)* let mut #ident: #ty = ::std::vec::Vec::new(); }
                    }
                    TypeKind::Other => {
                        quote! { #(#cfg)* let mut #ident: ::std::option::Option<#ty> = None; }
                    }
                }
            }
        })
        .collect();

    // --- Parsing loop arms for named unconditional fields ---
    let named_unconditional: Vec<&FieldConfig> = configs
        .iter()
        .filter(|f| f.is_named() && f.cfg_attrs.is_empty())
        .collect();

    let named_unconditional_arms: Vec<TokenStream2> = named_unconditional
        .iter()
        .map(|f| gen_named_arm(f))
        .collect();

    // --- cfg-conditional named fields (go in else block) ---
    let cfg_named: Vec<&FieldConfig> = configs
        .iter()
        .filter(|f| f.is_named() && !f.cfg_attrs.is_empty())
        .collect();

    let cfg_else_body: Vec<TokenStream2> = cfg_named
        .iter()
        .map(|f| {
            let cfg = &f.cfg_attrs;
            let conds = f.match_conditions();
            let cond = or_conditions(&conds);
            let set_body = gen_named_set_body(f);
            quote! {
                #(#cfg)*
                if !_handled && (#cond) {
                    #set_body
                    _handled = true;
                }
            }
        })
        .collect();

    let has_cfg_fields = !cfg_named.is_empty();
    let cfg_else_block: TokenStream2 = if has_cfg_fields {
        quote! {
            else {
                let mut _handled = false;
                #(#cfg_else_body)*
                if !_handled {
                    ::std::eprintln!("Unknown argument: {_arg}");
                    ::std::process::exit(1);
                }
            }
        }
    } else {
        quote! {
            else {
                ::std::eprintln!("Unknown argument: {_arg}");
                ::std::process::exit(1);
            }
        }
    };

    // --- Positional / subcommand block ---
    let subcommand_fields: Vec<&FieldConfig> = configs.iter().filter(|f| f.is_subcommand).collect();
    let positional_fields: Vec<&FieldConfig> = configs.iter().filter(|f| f.is_positional()).collect();

    let non_flag_block = gen_non_flag_block(&subcommand_fields, &positional_fields);

    // --- Version arm ---
    let version_arm = if cmd.version {
        quote! {
            else if _arg == "-V" || _arg == "--version" {
                println!("{}", env!("CARGO_PKG_VERSION"));
                ::std::process::exit(0);
            }
        }
    } else {
        quote! {}
    };

    // --- Construction ---
    let construction: Vec<TokenStream2> = configs
        .iter()
        .map(gen_field_construction)
        .collect();

    TokenStream::from(quote! {
        impl #name {
            /// Parse from a slice of argument strings.
            pub fn parse_from(_raw: &[::std::string::String]) -> Self {
                #(#decls)*

                let mut _pos = 0usize;
                while _pos < _raw.len() {
                    let _arg = &_raw[_pos];

                    if _arg == "-h" || _arg == "--help" {
                        #help_print
                        ::std::process::exit(0);
                    }
                    #version_arm
                    #(#named_unconditional_arms)*
                    #non_flag_block
                    #cfg_else_block

                    _pos += 1;
                }

                Self { #(#construction)* }
            }

            /// Parse from `std::env::args()`, skipping the binary name.
            pub fn parse() -> Self {
                Self::parse_from(&::std::env::args().skip(1).collect::<::std::vec::Vec<_>>())
            }

            /// Parse as a Cargo subcommand: skip the binary name and the first
            /// non-flag argument (e.g. `"bundle"` in `cargo bundle [args]`).
            pub fn cargo_parse() -> Self {
                let mut _args: ::std::vec::Vec<::std::string::String> =
                    ::std::env::args().skip(1).collect();
                if let Some(_pos) = _args.iter().position(|a| !a.starts_with('-')) {
                    _args.remove(_pos);
                }
                Self::parse_from(&_args)
            }
        }
    })
}

// Generate the else-if arm for a named field (including inline short check)
fn gen_named_arm(f: &FieldConfig) -> TokenStream2 {
    let conds = f.match_conditions();
    let cond = or_conditions(&conds);
    let set_body = gen_named_set_body(f);
    let cfg = &f.cfg_attrs;

    let inline_short = if let Some(c) = f.short {
        let kind = classify(&f.ty);
        if kind != TypeKind::Bool {
            let prefix = format!("-{c}");
            let set_inline = gen_named_set_inline(f);
            quote! {
                else if _arg.starts_with(#prefix) && _arg.len() > 2 {
                    #set_inline
                }
            }
        } else {
            quote! {}
        }
    } else {
        quote! {}
    };

    quote! {
        #(#cfg)*
        else if #cond {
            #set_body
        }
        #inline_short
    }
}

// Generate the body that reads and assigns the value for a named field
fn gen_named_set_body(f: &FieldConfig) -> TokenStream2 {
    let ident = &f.ident;
    let ty = &f.ty;
    let kind = classify(ty);

    match kind {
        TypeKind::Bool => quote! { #ident = true; },
        TypeKind::Str => {
            quote! {
                _pos += 1;
                #ident = Some(if let Some(_v) = _raw.get(_pos) {
                    _v.clone()
                } else {
                    ::std::eprintln!("Expected value after {_arg}");
                    ::std::process::exit(1);
                });
            }
        }
        TypeKind::OptionStr => {
            quote! {
                _pos += 1;
                #ident = Some(if let Some(_v) = _raw.get(_pos) {
                    _v.clone()
                } else {
                    ::std::eprintln!("Expected value after {_arg}");
                    ::std::process::exit(1);
                });
            }
        }
        TypeKind::OptionOther => {
            let inner = inner_ty(ty);
            quote! {
                _pos += 1;
                #ident = Some(if let Some(_v) = _raw.get(_pos) {
                    match _v.parse::<#inner>() {
                        Ok(_val) => _val,
                        Err(_) => {
                            ::std::eprintln!("Invalid value for {_arg}: {_v}");
                            ::std::process::exit(1);
                        }
                    }
                } else {
                    ::std::eprintln!("Expected value after {_arg}");
                    ::std::process::exit(1);
                });
            }
        }
        TypeKind::VecStr => {
            quote! {
                _pos += 1;
                #ident.push(if let Some(_v) = _raw.get(_pos) {
                    _v.clone()
                } else {
                    ::std::eprintln!("Expected value after {_arg}");
                    ::std::process::exit(1);
                });
            }
        }
        TypeKind::VecOther => {
            let inner = inner_ty(ty);
            quote! {
                _pos += 1;
                #ident.push(if let Some(_v) = _raw.get(_pos) {
                    match _v.parse::<#inner>() {
                        Ok(_val) => _val,
                        Err(_) => {
                            ::std::eprintln!("Invalid value for {_arg}: {_v}");
                            ::std::process::exit(1);
                        }
                    }
                } else {
                    ::std::eprintln!("Expected value after {_arg}");
                    ::std::process::exit(1);
                });
            }
        }
        TypeKind::Other => {
            quote! {
                _pos += 1;
                #ident = Some(if let Some(_v) = _raw.get(_pos) {
                    match _v.parse::<#ty>() {
                        Ok(_val) => _val,
                        Err(_) => {
                            ::std::eprintln!("Invalid value for {_arg}: {_v}");
                            ::std::process::exit(1);
                        }
                    }
                } else {
                    ::std::eprintln!("Expected value after {_arg}");
                    ::std::process::exit(1);
                });
            }
        }
    }
}

// Generate inline short value assignment: -Xpath -> field = "xpath"
fn gen_named_set_inline(f: &FieldConfig) -> TokenStream2 {
    let ident = &f.ident;
    let ty = &f.ty;
    let kind = classify(ty);
    match kind {
        TypeKind::Str => quote! { #ident = Some(_arg[2..].to_string()); },
        TypeKind::OptionStr => quote! { #ident = Some(_arg[2..].to_string()); },
        TypeKind::VecStr => quote! { #ident.push(_arg[2..].to_string()); },
        TypeKind::OptionOther => {
            let inner = inner_ty(ty);
            quote! {
                match _arg[2..].parse::<#inner>() {
                    Ok(_val) => #ident = Some(_val),
                    Err(_) => {
                        ::std::eprintln!("Invalid value: {}", &_arg[2..]);
                        ::std::process::exit(1);
                    }
                }
            }
        }
        TypeKind::VecOther => {
            let inner = inner_ty(ty);
            quote! {
                match _arg[2..].parse::<#inner>() {
                    Ok(_val) => #ident.push(_val),
                    Err(_) => {
                        ::std::eprintln!("Invalid value: {}", &_arg[2..]);
                        ::std::process::exit(1);
                    }
                }
            }
        }
        _ => quote! { #ident = Some(_arg[2..].to_string()); },
    }
}

// Generate the else-if block for non-flag args (subcommand + positionals)
fn gen_non_flag_block(
    subcommand_fields: &[&FieldConfig],
    positional_fields: &[&FieldConfig],
) -> TokenStream2 {
    if subcommand_fields.is_empty() && positional_fields.is_empty() {
        return quote! {};
    }

    let mut stmts: Vec<TokenStream2> = Vec::new();
    stmts.push(quote! { let mut _pos_handled = false; });

    // Try subcommand fields
    for f in subcommand_fields {
        let ident = &f.ident;
        let ty = &f.ty;
        let kind = classify(ty);
        let inner = if matches!(kind, TypeKind::OptionOther | TypeKind::OptionStr) {
            inner_ty(ty)
        } else {
            quote!(#ty)
        };
        stmts.push(quote! {
            if !_pos_handled && #ident.is_none() {
                if let Some(_sc) = <#inner as clap::__private::Subcommand>::try_parse(_arg, _raw, &mut _pos) {
                    #ident = Some(_sc);
                    _pos_handled = true;
                }
            }
        });
    }

    // Try positional fields in order
    for f in positional_fields {
        let ident = &f.ident;
        let ty = &f.ty;
        let kind = classify(ty);
        let stmt = match kind {
            TypeKind::VecStr => quote! {
                if !_pos_handled {
                    #ident.push(_arg.clone());
                    _pos_handled = true;
                }
            },
            TypeKind::VecOther => {
                let inner = inner_ty(ty);
                quote! {
                    if !_pos_handled {
                        match _arg.parse::<#inner>() {
                            Ok(_val) => #ident.push(_val),
                            Err(_) => {
                                ::std::eprintln!("Invalid positional value: {_arg}");
                                ::std::process::exit(1);
                            }
                        }
                        _pos_handled = true;
                    }
                }
            }
            TypeKind::OptionStr => quote! {
                if !_pos_handled && #ident.is_none() {
                    #ident = Some(_arg.clone());
                    _pos_handled = true;
                }
            },
            TypeKind::OptionOther => {
                let inner = inner_ty(ty);
                quote! {
                    if !_pos_handled && #ident.is_none() {
                        match _arg.parse::<#inner>() {
                            Ok(_val) => #ident = Some(_val),
                            Err(_) => {
                                ::std::eprintln!("Invalid positional value: {_arg}");
                                ::std::process::exit(1);
                            }
                        }
                        _pos_handled = true;
                    }
                }
            }
            TypeKind::Str | TypeKind::Other => quote! {
                if !_pos_handled && #ident.is_none() {
                    #ident = Some(_arg.clone());
                    _pos_handled = true;
                }
            },
            TypeKind::Bool => quote! {
                if !_pos_handled {
                    _pos_handled = true;
                }
            },
        };
        stmts.push(stmt);
    }

    stmts.push(quote! {
        if !_pos_handled {
            ::std::eprintln!("Unknown argument: {_arg}");
            ::std::process::exit(1);
        }
    });

    let result: TokenStream2 = quote! {
        else if !_arg.starts_with('-') {
            #(#stmts)*
        }
    };
    result
}

// Generate the Self { field: value, ... } construction for one field
fn gen_field_construction(f: &FieldConfig) -> TokenStream2 {
    let ident = &f.ident;
    let ty = &f.ty;
    let kind = classify(ty);
    let cfg = &f.cfg_attrs;

    if f.is_subcommand {
        // If field is Option<T>, pass as-is; if T, unwrap (show help on None)
        if matches!(kind, TypeKind::OptionOther | TypeKind::OptionStr) {
            quote! { #(#cfg)* #ident, }
        } else {
            // non-Option subcommand: unwrap or show help
            quote! {
                #(#cfg)*
                #ident: #ident.unwrap_or_else(|| {
                    ::std::eprintln!("error: missing required subcommand");
                    ::std::process::exit(1);
                }),
            }
        }
    } else if f.is_positional() {
        match kind {
            TypeKind::VecStr | TypeKind::VecOther => quote! { #(#cfg)* #ident, },
            TypeKind::OptionStr | TypeKind::OptionOther => quote! { #(#cfg)* #ident, },
            TypeKind::Bool => quote! { #(#cfg)* #ident, },
            TypeKind::Str | TypeKind::Other => {
                if let Some(ref dv) = f.default_value {
                    quote! { #(#cfg)* #ident: #ident.unwrap_or_else(|| #dv.to_string()), }
                } else {
                    quote! { #(#cfg)* #ident: #ident.unwrap_or_default(), }
                }
            }
        }
    } else {
        // Named field
        match kind {
            TypeKind::Bool => quote! { #(#cfg)* #ident, },
            TypeKind::Str => {
                if let Some(ref dv) = f.default_value {
                    quote! { #(#cfg)* #ident: #ident.unwrap_or_else(|| #dv.to_string()), }
                } else {
                    quote! { #(#cfg)* #ident: #ident.unwrap_or_default(), }
                }
            }
            TypeKind::OptionStr | TypeKind::OptionOther => quote! { #(#cfg)* #ident, },
            TypeKind::VecStr | TypeKind::VecOther => quote! { #(#cfg)* #ident, },
            TypeKind::Other => {
                if let Some(ref dv) = f.default_value {
                    // Parse the default value string
                    quote! {
                        #(#cfg)*
                        #ident: #ident.unwrap_or_else(|| {
                            #dv.parse().expect("Invalid default_value")
                        }),
                    }
                } else {
                    quote! { #(#cfg)* #ident: #ident.unwrap_or_default(), }
                }
            }
        }
    }
}

// MARK: Subcommand derive

struct VariantConfig {
    ident: syn::Ident,
    name: String,
    aliases: Vec<String>,
    fields: Vec<VariantField>,
}

struct VariantField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
}

impl VariantConfig {
    fn from_variant(variant: &syn::Variant) -> Self {
        let ident = variant.ident.clone();
        let default_name = camel_to_kebab(&ident.to_string());

        let mut name = default_name;
        let mut aliases = Vec::new();

        for attr in &variant.attrs {
            if attr.path().is_ident("command") {
                let items = attr
                    .parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
                    .expect("Invalid #[command] attribute");
                for item in &items {
                    match item {
                        Meta::NameValue(nv) if nv.path.is_ident("name") => {
                            if let Some(s) = str_from_meta(item) {
                                name = s;
                            }
                        }
                        Meta::NameValue(nv) if nv.path.is_ident("alias") => {
                            if let Some(s) = str_from_meta(item) {
                                aliases.push(s);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        let fields: Vec<VariantField> = match &variant.fields {
            syn::Fields::Named(named) => named
                .named
                .iter()
                .map(|f| VariantField {
                    ident: f.ident.clone(),
                    ty: f.ty.clone(),
                })
                .collect(),
            syn::Fields::Unnamed(unnamed) => unnamed
                .unnamed
                .iter()
                .map(|f| VariantField {
                    ident: None,
                    ty: f.ty.clone(),
                })
                .collect(),
            syn::Fields::Unit => vec![],
        };

        VariantConfig {
            ident,
            name,
            aliases,
            fields,
        }
    }
}

/// `#[derive(Subcommand)]` — generates `try_parse` for subcommand enums.
#[proc_macro_derive(Subcommand, attributes(command))]
pub fn subcommand_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let variants: Vec<VariantConfig> = match &input.data {
        syn::Data::Enum(data) => data.variants.iter().map(VariantConfig::from_variant).collect(),
        _ => panic!("Subcommand can only be derived for enums"),
    };

    let match_arms: Vec<TokenStream2> = variants.iter().map(|v| {
        let variant_ident = &v.ident;
        let primary = &v.name;
        let all_names: Vec<&str> = std::iter::once(primary.as_str())
            .chain(v.aliases.iter().map(String::as_str))
            .collect();

        if v.fields.is_empty() {
            // Unit variant
            quote! {
                #(#all_names)|* => ::std::option::Option::Some(Self::#variant_ident),
            }
        } else {
            // Struct/tuple variant with fields
            let field_decls: Vec<TokenStream2> = v.fields.iter().enumerate().map(|(i, f)| {
                let field_ident = f.ident.clone().unwrap_or_else(|| {
                    syn::Ident::new(&format!("_f{i}"), Span::call_site())
                });
                let ty = &f.ty;
                let kind = classify(ty);
                match kind {
                    TypeKind::OptionStr | TypeKind::OptionOther => {
                        quote! { let mut #field_ident: #ty = None; }
                    }
                    TypeKind::Str => {
                        quote! { let mut #field_ident: ::std::option::Option<::std::string::String> = None; }
                    }
                    _ => {
                        quote! { let mut #field_ident: ::std::option::Option<#ty> = None; }
                    }
                }
            }).collect();

            // For each field, try to read from the next positional arg
            let field_reads: Vec<TokenStream2> = v.fields.iter().enumerate().map(|(i, f)| {
                let field_ident = f.ident.clone().unwrap_or_else(|| {
                    syn::Ident::new(&format!("_f{i}"), Span::call_site())
                });
                let kind = classify(&f.ty);
                match kind {
                    TypeKind::Str | TypeKind::OptionStr => {
                        quote! {
                            if let Some(_next) = args.get(*pos + 1) {
                                if !_next.starts_with('-') {
                                    #field_ident = Some(_next.clone());
                                    *pos += 1;
                                }
                            }
                        }
                    }
                    TypeKind::OptionOther => {
                        let inner = inner_ty(&f.ty);
                        quote! {
                            if let Some(_next) = args.get(*pos + 1) {
                                if !_next.starts_with('-') {
                                    if let Ok(_val) = _next.parse::<#inner>() {
                                        #field_ident = Some(_val);
                                        *pos += 1;
                                    }
                                }
                            }
                        }
                    }
                    _ => quote! {},
                }
            }).collect();

            let field_constructs: Vec<TokenStream2> = v.fields.iter().enumerate().map(|(i, f)| {
                let field_ident = f.ident.clone().unwrap_or_else(|| {
                    syn::Ident::new(&format!("_f{i}"), Span::call_site())
                });
                let kind = classify(&f.ty);
                if matches!(kind, TypeKind::OptionStr | TypeKind::OptionOther) {
                    quote! { #field_ident, }
                } else {
                    quote! { #field_ident: #field_ident.unwrap_or_default(), }
                }
            }).collect();

            let has_named = v.fields.iter().any(|f| f.ident.is_some());
            let construct: TokenStream2 = if has_named {
                quote! { Self::#variant_ident { #(#field_constructs)* } }
            } else {
                // Tuple variant - not used in current binaries but handle anyway
                let vals: Vec<TokenStream2> = v.fields.iter().enumerate().map(|(i, f)| {
                    let id = syn::Ident::new(&format!("_f{i}"), Span::call_site());
                    let kind = classify(&f.ty);
                    if matches!(kind, TypeKind::OptionStr | TypeKind::OptionOther) {
                        quote! { #id }
                    } else {
                        quote! { #id.unwrap_or_default() }
                    }
                }).collect();
                quote! { Self::#variant_ident(#(#vals),*) }
            };

            let arm: TokenStream2 = quote! {
                #(#all_names)|* => {
                    #(#field_decls)*
                    #(#field_reads)*
                    ::std::option::Option::Some(#construct)
                }
            };
            arm
        }
    }).collect();

    TokenStream::from(quote! {
        impl clap::__private::Subcommand for #name {
            fn try_parse(name: &str, args: &[::std::string::String], pos: &mut usize) -> ::std::option::Option<Self> {
                match name {
                    #(#match_arms)*
                    _ => ::std::option::Option::None,
                }
            }
        }
    })
}
