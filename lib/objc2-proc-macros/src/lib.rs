/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Proc-macro support for the objc2 crate: provides the `define_class!` macro.

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{
    Attribute, FnArg, Ident, ImplItem, ItemImpl, ItemStruct, Meta, Token, parse_macro_input,
};

struct DefineClassInput {
    struct_item: ItemStruct,
    impl_item: Option<ItemImpl>,
}

impl Parse for DefineClassInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let struct_item: ItemStruct = input.parse()?;
        let impl_item = if !input.is_empty() {
            Some(input.parse()?)
        } else {
            None
        };
        Ok(Self {
            struct_item,
            impl_item,
        })
    }
}

fn extract_super(attrs: &[Attribute]) -> Option<syn::Path> {
    for attr in attrs {
        if let Meta::List(ml) = &attr.meta
            && ml.path.is_ident("unsafe")
        {
            let result =
                ml.parse_args_with(|input: ParseStream| -> syn::Result<Option<syn::Path>> {
                    if input.peek(Token![super]) {
                        input.parse::<Token![super]>()?;
                        let content;
                        syn::parenthesized!(content in input);
                        let path: syn::Path = content.parse()?;
                        return Ok(Some(path));
                    }
                    Err(input.error(""))
                });
            if let Ok(Some(path)) = result {
                return Some(path);
            }
        }
    }
    None
}

fn extract_name(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if let Meta::NameValue(mnv) = &attr.meta
            && mnv.path.is_ident("name")
            && let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) = &mnv.value
        {
            return Some(s.value());
        }
    }
    None
}

fn extract_ivars(attrs: &[Attribute]) -> Option<syn::Expr> {
    for attr in attrs {
        if let Meta::NameValue(mnv) = &attr.meta
            && mnv.path.is_ident("ivars")
        {
            return Some(mnv.value.clone());
        }
    }
    None
}

fn extract_selector(attr: &Attribute) -> Option<String> {
    if let Meta::List(ml) = &attr.meta
        && ml.path.is_ident("unsafe")
    {
        let result = ml.parse_args_with(|input: ParseStream| -> syn::Result<Option<String>> {
            if input.peek(Ident) {
                let kw: Ident = input.call(IdentExt::parse_any)?;
                if kw == "method" {
                    let content;
                    syn::parenthesized!(content in input);
                    let mut sel = String::new();
                    while !content.is_empty() {
                        if content.peek(Token![:]) {
                            content.parse::<Token![:]>()?;
                            sel.push(':');
                        } else {
                            let part: Ident = content.call(IdentExt::parse_any)?;
                            sel.push_str(&part.to_string());
                            if content.peek(Token![:]) {
                                content.parse::<Token![:]>()?;
                                sel.push(':');
                            }
                        }
                    }
                    return Ok(Some(sel));
                }
            }
            Err(input.error(""))
        });
        if let Ok(Some(s)) = result {
            return Some(s);
        }
    }
    None
}

fn sel_tokens(selector: &str) -> impl quote::ToTokens {
    if !selector.contains(':') {
        let ident = format_ident!("{selector}");
        quote! { ::objc2::sel!(#ident) }
    } else {
        let parts: Vec<Ident> = selector
            .split(':')
            .filter(|s| !s.is_empty())
            .map(|s| format_ident!("{s}"))
            .collect();
        quote! { ::objc2::sel!(#(#parts:)*) }
    }
}

/// Declares a Rust type for an existing Objective-C class, matching upstream objc2 0.6 syntax.
///
/// # Syntax
///
/// ```text
/// extern_class!(
///     #[unsafe(super(NSObject))]
///     #[name = "ObjCClassName"]   // optional, defaults to struct ident
///     pub struct MyClass;
/// );
/// ```
///
/// Generates a `#[repr(C)]` struct and a `class()` method that returns the existing ObjC class.
#[proc_macro]
pub fn extern_class(input: TokenStream) -> TokenStream {
    let struct_item: ItemStruct = parse_macro_input!(input as ItemStruct);

    let struct_name = &struct_item.ident;
    let struct_vis = &struct_item.vis;
    let class_name = extract_name(&struct_item.attrs).unwrap_or_else(|| struct_name.to_string());
    let class_ident = format_ident!("{class_name}");

    let forwarded_attrs: Vec<&Attribute> = struct_item
        .attrs
        .iter()
        .filter(|a| {
            let p = a.meta.path();
            !p.is_ident("unsafe") && !p.is_ident("name")
        })
        .collect();

    quote! {
        #(#forwarded_attrs)*
        #[repr(C)]
        #struct_vis struct #struct_name {
            _super: ::objc2::runtime::AnyObject,
        }

        impl #struct_name {
            fn class() -> *mut ::objc2::runtime::AnyObject {
                ::objc2::class!(#class_ident)
            }
        }
    }
    .into()
}

/// Defines an Objective-C class using declarative syntax matching upstream objc2 0.6.
///
/// # Syntax
///
/// ```text
/// define_class!(
///     #[unsafe(super(SuperClass))]
///     #[name = "ObjCClassName"]   // optional, defaults to struct ident
///     #[ivars = IvarsType]        // optional
///     struct MyClass;
///
///     impl MyClass {
///         #[unsafe(method(selector:parts:))]
///         fn my_method(&self, arg: Type) -> RetType { ... }
///     }
/// );
/// ```
///
/// Generates a `#[repr(C)]` struct, `extern "C"` trampolines, a lazy `class()` registration
/// method, and an `ivars()` accessor (when `#[ivars]` is present).
#[proc_macro]
pub fn define_class(input: TokenStream) -> TokenStream {
    let DefineClassInput {
        struct_item,
        impl_item,
    } = parse_macro_input!(input as DefineClassInput);

    let struct_name = &struct_item.ident;
    let struct_vis = &struct_item.vis;

    let super_path: syn::Path =
        extract_super(&struct_item.attrs).unwrap_or_else(|| syn::parse_quote!(NSObject));
    let class_name = extract_name(&struct_item.attrs).unwrap_or_else(|| struct_name.to_string());
    let ivars_expr = extract_ivars(&struct_item.attrs);

    let forwarded_attrs: Vec<&Attribute> = struct_item
        .attrs
        .iter()
        .filter(|a| {
            let p = a.meta.path();
            !p.is_ident("unsafe") && !p.is_ident("name") && !p.is_ident("ivars")
        })
        .collect();

    let mut trampolines = Vec::new();
    let mut user_methods = Vec::new();
    let mut registrations = Vec::new();

    if let Some(ref impl_block) = impl_item {
        for item in &impl_block.items {
            if let ImplItem::Fn(method) = item {
                let selector = method.attrs.iter().find_map(extract_selector);
                if let Some(sel_str) = selector {
                    let fn_name = &method.sig.ident;
                    let trampoline_name = format_ident!(
                        "__trampoline_{}",
                        fn_name.to_string().trim_start_matches('_')
                    );
                    let sel_mac = sel_tokens(&sel_str);
                    let ret = &method.sig.output;

                    let typed_args: Vec<_> = method
                        .sig
                        .inputs
                        .iter()
                        .filter_map(|a| {
                            if let FnArg::Typed(pt) = a {
                                Some(pt)
                            } else {
                                None
                            }
                        })
                        .collect();
                    let arg_names: Vec<_> = (0..typed_args.len())
                        .map(|i| format_ident!("__arg{i}"))
                        .collect();
                    let arg_types: Vec<_> = typed_args.iter().map(|pt| &pt.ty).collect();

                    trampolines.push(quote! {
                        #[allow(clippy::undocumented_unsafe_blocks)]
                        extern "C" fn #trampoline_name(
                            __this: *mut ::objc2::runtime::AnyObject,
                            __sel: ::objc2::runtime::Sel,
                            #(#arg_names: #arg_types,)*
                        ) #ret {
                            unsafe { &*(__this as *const Self) }.#fn_name(#(#arg_names,)*)
                        }
                    });

                    let n_args = arg_types.len();
                    let wildcards: Vec<_> = (0..n_args).map(|_| quote! { _ }).collect();
                    registrations.push(quote! {
                        builder.add_method(
                            #sel_mac,
                            Self::#trampoline_name as extern "C" fn(_, _, #(#wildcards,)*) #ret,
                        );
                    });

                    let filtered_attrs: Vec<&Attribute> = method
                        .attrs
                        .iter()
                        .filter(|a| extract_selector(a).is_none())
                        .collect();
                    let vis = &method.vis;
                    let sig = &method.sig;
                    let body = &method.block;
                    user_methods.push(quote! {
                        #(#filtered_attrs)*
                        #vis #sig #body
                    });
                }
            }
        }
    }

    let ivar_reg = ivars_expr.as_ref().map(|ivars| {
        quote! {
            builder.add_ivar_raw::<#ivars>(unsafe {
                ::std::ffi::CStr::from_bytes_with_nul_unchecked(b"__ivars\0")
            });
        }
    });

    let ivars_method = ivars_expr.as_ref().map(|ivars| {
        quote! {
            #[allow(clippy::undocumented_unsafe_blocks)]
            fn ivars(&self) -> &#ivars {
                static OFFSET: ::std::sync::OnceLock<usize> = ::std::sync::OnceLock::new();
                let offset = *OFFSET.get_or_init(|| unsafe {
                    let ivar = ::objc2::ffi::class_getInstanceVariable(
                        Self::class() as *const _,
                        b"__ivars\0".as_ptr() as *const ::std::ffi::c_char,
                    );
                    assert!(!ivar.is_null(), "__ivars ivar not found on class");
                    ::objc2::ffi::ivar_getOffset(ivar) as usize
                });
                unsafe { &*((self as *const Self as *const u8).add(offset) as *const #ivars) }
            }
        }
    });

    let class_name_nul = format!("{class_name}\0");
    let class_method = quote! {
        #[allow(clippy::undocumented_unsafe_blocks)]
        fn class() -> *mut ::objc2::runtime::AnyObject {
            static CLASS: ::std::sync::OnceLock<usize> = ::std::sync::OnceLock::new();
            *CLASS.get_or_init(|| unsafe {
                let mut builder = ::objc2::runtime::ClassBuilder::new(
                    ::std::ffi::CStr::from_bytes_with_nul_unchecked(#class_name_nul.as_bytes()),
                    ::objc2::class!(#super_path),
                ).expect(concat!("class \"", #class_name, "\" already registered"));
                #ivar_reg
                #(#registrations)*
                builder.register() as usize
            }) as *mut ::objc2::runtime::AnyObject
        }
    };

    quote! {
        #(#forwarded_attrs)*
        #[repr(C)]
        #struct_vis struct #struct_name {
            _super: ::objc2::runtime::AnyObject,
        }

        impl #struct_name {
            #(#trampolines)*
            #(#user_methods)*
            #class_method
            #ivars_method
        }
    }
    .into()
}
