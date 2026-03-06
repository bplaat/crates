/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("README.md")]

use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");

    if env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set") == "windows" {
        generate_webview2_bindings(&manifest_dir, &out_dir);

        // Link with the correct WebView2Loader library based on architecture
        let target = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH not set");
        let lib_dir = PathBuf::from(&manifest_dir)
            .join("webview2")
            .join(if target == "x86_64" {
                "x64"
            } else if target == "aarch64" {
                "arm64"
            } else if target == "x86" {
                "x86"
            } else {
                panic!("Unsupported architecture")
            });

        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
        match target_env.as_str() {
            "msvc" => {
                println!("cargo:rustc-link-lib=static=WebView2LoaderStatic");
            }
            "gnu" => {
                println!("cargo:rustc-link-lib=dylib=WebView2Loader");

                // Copy WebView2Loader.dll to output directory for dynamic linking
                let out_dir_path = PathBuf::from(&out_dir);
                std::fs::copy(
                    lib_dir.join("WebView2Loader.dll"),
                    out_dir_path
                        .parent()
                        .expect("Should be some")
                        .parent()
                        .expect("Should be some")
                        .parent()
                        .expect("Should be some")
                        .join("WebView2Loader.dll"),
                )
                .expect("Failed to copy WebView2Loader.dll");
            }
            other => {
                panic!("unsupported target environment: {other}");
            }
        }
    }
}

// (method_name, params: Vec<(param_name, param_type)>, ret_type)
type ImplMethodData = (String, Vec<(String, String)>, String);

fn generate_webview2_bindings(manifest_dir: &str, out_dir: &str) {
    use std::collections::{HashMap, HashSet};
    use std::fmt::Write as _;

    use windows_metadata::Value;
    use windows_metadata::reader::{HasAttributes, Index, TypeCategory};

    let winmd_path = PathBuf::from(manifest_dir)
        .join("webview2")
        .join("Microsoft.Web.WebView2.Win32.winmd");
    println!("cargo:rerun-if-changed={}", winmd_path.display());

    let index = Index::read(&winmd_path).expect("Failed to read WebView2 winmd");

    // Pre-compute WebView2 enum underlying types (name → rust type)
    let enum_types: HashMap<String, String> = index
        .all()
        .filter(|t| {
            t.namespace() == "Microsoft.Web.WebView2.Win32" && t.category() == TypeCategory::Enum
        })
        .map(|t| {
            let underlying = t
                .fields()
                .filter(|f| f.name() != "value__")
                .find_map(|f| {
                    f.constant().map(|c| match c.value() {
                        Value::I32(_) => "i32",
                        Value::U32(_) => "u32",
                        Value::I64(_) => "i64",
                        Value::U64(_) => "u64",
                        _ => "i32",
                    })
                })
                .unwrap_or("i32")
                .to_string();
            (t.name().to_string(), underlying)
        })
        .collect();

    // Pre-compute struct type names (for value-type pass-by-value detection)
    let struct_types: HashSet<String> = index
        .all()
        .filter(|t| {
            t.namespace() == "Microsoft.Web.WebView2.Win32" && t.category() == TypeCategory::Struct
        })
        .map(|t| t.name().to_string())
        .collect();

    // Pre-compute interface inheritance: interface_name -> parent_interface_name (WebView2 only)
    let interface_parent: HashMap<String, String> = index
        .all()
        .filter(|t| {
            t.namespace() == "Microsoft.Web.WebView2.Win32"
                && t.category() == TypeCategory::Interface
        })
        .filter_map(|t| {
            let parent_name = t
                .interface_impls()
                .find_map(|impl_| match impl_.interface(&[]) {
                    windows_metadata::Type::Name(tn)
                        if tn.namespace == "Microsoft.Web.WebView2.Win32" =>
                    {
                        Some(tn.name.clone())
                    }
                    _ => None,
                });
            parent_name.map(|p| (t.name().to_string(), p))
        })
        .collect();

    // Pre-compute vtable entry strings per interface (own methods only, no visibility prefix, no indent)
    // Format: "MethodName: unsafe extern \"system\" fn(This: *mut OwnerType, ...) -> Ret,"
    let interface_vtbl_entries: HashMap<String, Vec<String>> = {
        let mut map = HashMap::new();
        for t in index.all().filter(|t| {
            t.namespace() == "Microsoft.Web.WebView2.Win32"
                && t.category() == TypeCategory::Interface
        }) {
            let entries: Vec<String> = t
                .methods()
                .map(|m| {
                    let sig = m.signature(&[]);
                    let params: Vec<_> = m.params().skip(1).collect();
                    let ret = map_type(&sig.return_type, &enum_types, &struct_types);
                    let mut entry = format!(
                        "{}: unsafe extern \"system\" fn(This: *mut {}",
                        m.name(),
                        t.name()
                    );
                    for (i, ty) in sig.types.iter().enumerate() {
                        let name = if i < params.len() && !params[i].name().is_empty() {
                            params[i].name().to_string()
                        } else {
                            format!("p{i}")
                        };
                        entry.push_str(&format!(
                            ", {}: {}",
                            name,
                            map_type(ty, &enum_types, &struct_types)
                        ));
                    }
                    entry.push_str(&format!(") -> {ret},"));
                    entry
                })
                .collect();
            map.insert(t.name().to_string(), entries);
        }
        map
    };

    // Pre-compute impl method data per interface (own methods only)
    let interface_impl_methods: HashMap<String, Vec<ImplMethodData>> = {
        let mut map = HashMap::new();
        for t in index.all().filter(|t| {
            t.namespace() == "Microsoft.Web.WebView2.Win32"
                && t.category() == TypeCategory::Interface
        }) {
            let entries: Vec<ImplMethodData> = t
                .methods()
                .map(|m| {
                    let sig = m.signature(&[]);
                    let params: Vec<_> = m.params().skip(1).collect();
                    let ret = map_type(&sig.return_type, &enum_types, &struct_types);
                    let param_pairs: Vec<(String, String)> = sig
                        .types
                        .iter()
                        .enumerate()
                        .map(|(i, ty)| {
                            let name = if i < params.len() && !params[i].name().is_empty() {
                                params[i].name().to_string()
                            } else {
                                format!("p{i}")
                            };
                            (name, map_type(ty, &enum_types, &struct_types))
                        })
                        .collect();
                    (m.name().to_string(), param_pairs, ret)
                })
                .collect();
            map.insert(t.name().to_string(), entries);
        }
        map
    };

    let mut code = String::new();
    _ = writeln!(
        code,
        "// Auto-generated by build.rs from Microsoft.Web.WebView2.Win32.winmd — do not edit.\n"
    );
    _ = writeln!(code, "use std::ffi::c_void;");
    _ = writeln!(code, "use super::win32::*;");
    _ = writeln!(code);

    // EventRegistrationToken (WinRT type used for event subscriptions)
    _ = writeln!(code, "#[repr(C)]");
    _ = writeln!(code, "pub(crate) struct EventRegistrationToken {{");
    _ = writeln!(code, "    pub(crate) value: i64,");
    _ = writeln!(code, "}}");
    _ = writeln!(code);

    // Struct value types
    for t in index.all().filter(|t| {
        t.namespace() == "Microsoft.Web.WebView2.Win32" && t.category() == TypeCategory::Struct
    }) {
        _ = writeln!(code, "#[repr(C)]");
        _ = writeln!(code, "#[allow(dead_code)]");
        _ = writeln!(code, "pub(crate) struct {} {{", t.name());
        for f in t.fields() {
            let field_type = map_type(&f.ty(), &enum_types, &struct_types);
            _ = writeln!(code, "    pub(crate) {}: {},", f.name(), field_type);
        }
        _ = writeln!(code, "}}");
        _ = writeln!(code);
    }

    // Extern function block (Apis class = PInvoke methods)
    for t in index.all().filter(|t| {
        t.namespace() == "Microsoft.Web.WebView2.Win32"
            && t.category() == TypeCategory::Class
            && t.name() == "Apis"
    }) {
        _ = writeln!(
            code,
            "#[cfg_attr(not(target_env = \"msvc\"), link(name = \"WebView2Loader\"))]"
        );
        _ = writeln!(code, "unsafe extern \"system\" {{");
        for m in t.methods() {
            let sig = m.signature(&[]);
            let params: Vec<_> = m.params().skip(1).collect();
            let ret = map_type(&sig.return_type, &enum_types, &struct_types);
            _ = write!(code, "    pub(crate) fn {}(", m.name());
            let param_strs: Vec<String> = sig
                .types
                .iter()
                .enumerate()
                .map(|(i, ty)| {
                    let name = if i < params.len() && !params[i].name().is_empty() {
                        params[i].name().to_string()
                    } else {
                        format!("p{i}")
                    };
                    format!("{}: {}", name, map_type(ty, &enum_types, &struct_types))
                })
                .collect();
            _ = write!(code, "{}", param_strs.join(", "));
            _ = writeln!(code, ") -> {ret};");
        }
        _ = writeln!(code, "}}");
        _ = writeln!(code);
    }

    // Enum constants
    for t in index.all().filter(|t| {
        t.namespace() == "Microsoft.Web.WebView2.Win32" && t.category() == TypeCategory::Enum
    }) {
        let underlying = enum_types
            .get(t.name())
            .map(|s| s.as_str())
            .unwrap_or("i32");
        for f in t.fields() {
            if f.name() == "value__" {
                continue;
            }
            if let Some(c) = f.constant() {
                let val: i64 = match c.value() {
                    Value::I32(v) => v as i64,
                    Value::U32(v) => v as i64,
                    Value::I64(v) => v,
                    Value::U64(v) => v as i64,
                    _ => 0,
                };
                _ = writeln!(code, "#[allow(dead_code)]",);
                _ = writeln!(
                    code,
                    "pub(crate) const {}: {} = {};",
                    f.name(),
                    underlying,
                    val
                );
            }
        }
    }
    _ = writeln!(code);

    // COM interfaces
    for t in index.all().filter(|t| {
        t.namespace() == "Microsoft.Web.WebView2.Win32" && t.category() == TypeCategory::Interface
    }) {
        let is_handler = t.name().ends_with("Handler");

        // IID constant (if interface has a GUID attribute)
        if let Some(attr) = t.find_attribute("GuidAttribute") {
            let args = attr.value();
            if args.len() == 11 {
                let data1 = match args[0].1 {
                    Value::U32(v) => v,
                    _ => 0,
                };
                let data2 = match args[1].1 {
                    Value::U16(v) => v,
                    _ => 0,
                };
                let data3 = match args[2].1 {
                    Value::U16(v) => v,
                    _ => 0,
                };
                let data4: Vec<u8> = args[3..11]
                    .iter()
                    .map(|(_, v)| match v {
                        Value::U8(b) => *b,
                        _ => 0,
                    })
                    .collect();
                _ = writeln!(code, "pub(crate) const IID_{}: GUID = GUID {{", t.name());
                _ = writeln!(code, "    data1: 0x{data1:08x},");
                _ = writeln!(code, "    data2: 0x{data2:04x},");
                _ = writeln!(code, "    data3: 0x{data3:04x},");
                let bytes_str: Vec<String> = data4.iter().map(|b| format!("0x{b:02x}")).collect();
                _ = writeln!(code, "    data4: [{}],", bytes_str.join(", "));
                _ = writeln!(code, "}};");
            }
        }

        // Struct definition
        _ = writeln!(code, "#[repr(C)]");
        _ = writeln!(code, "pub(crate) struct {} {{", t.name());
        _ = writeln!(code, "    pub(crate) lpVtbl: *const {}Vtbl,", t.name());
        if is_handler {
            _ = writeln!(code, "    pub(crate) user_data: *mut c_void,");
        }
        _ = writeln!(code, "}}");
        _ = writeln!(code);

        // impl block with wrapper methods (non-handlers only)
        let ancestor_impl_methods =
            collect_ancestor_impl_methods(t.name(), &interface_parent, &interface_impl_methods);
        let methods: Vec<_> = t.methods().collect();
        if !is_handler && (!methods.is_empty() || !ancestor_impl_methods.is_empty()) {
            _ = writeln!(code, "impl {} {{", t.name());
            // IUnknown base methods
            _ = writeln!(
                code,
                "    pub(crate) unsafe fn QueryInterface(&self, riid: *const GUID, ppvObject: *mut *mut c_void) -> HRESULT {{"
            );
            _ = writeln!(
                code,
                "        unsafe {{ ((*self.lpVtbl).QueryInterface)(self as *const _ as *mut _, riid, ppvObject) }}"
            );
            _ = writeln!(code, "    }}");
            _ = writeln!(code, "    pub(crate) unsafe fn AddRef(&self) -> HRESULT {{");
            _ = writeln!(
                code,
                "        unsafe {{ ((*self.lpVtbl).AddRef)(self as *const _ as *mut _) }}"
            );
            _ = writeln!(code, "    }}");
            _ = writeln!(
                code,
                "    pub(crate) unsafe fn Release(&self) -> HRESULT {{"
            );
            _ = writeln!(
                code,
                "        unsafe {{ ((*self.lpVtbl).Release)(self as *const _ as *mut _) }}"
            );
            _ = writeln!(code, "    }}");
            // Emit a single wrapper method
            let emit_wrapper =
                |code: &mut String, mname: &str, mparams: &[(String, String)], mret: &str| {
                    _ = write!(code, "    pub(crate) unsafe fn {mname}(&self");
                    for (pname, ptype) in mparams {
                        _ = write!(code, ", {pname}: {ptype}");
                    }
                    _ = writeln!(code, ") -> {mret} {{");
                    _ = write!(
                        code,
                        "        unsafe {{ ((*self.lpVtbl).{mname})(self as *const _ as *mut _"
                    );
                    for (pname, _) in mparams {
                        _ = write!(code, ", {pname}");
                    }
                    _ = writeln!(code, ") }}");
                    _ = writeln!(code, "    }}");
                };
            // Inherited wrappers (ancestor interfaces, root-first order)
            for (mname, mparams, mret) in &ancestor_impl_methods {
                emit_wrapper(&mut code, mname, mparams, mret);
            }
            // Own wrappers
            let own_impl: Vec<ImplMethodData> = interface_impl_methods
                .get(t.name())
                .cloned()
                .unwrap_or_default();
            for (mname, mparams, mret) in &own_impl {
                emit_wrapper(&mut code, mname, mparams, mret);
            }
            _ = writeln!(code, "}}");
            _ = writeln!(code);
        }

        // Vtbl struct — must include all inherited method slots before own methods
        let ancestor_entries =
            collect_ancestor_vtbl_entries(t.name(), &interface_parent, &interface_vtbl_entries);
        let vis = if is_handler { "pub(crate) " } else { "" };
        _ = writeln!(code, "#[repr(C)]");
        _ = writeln!(code, "pub(crate) struct {}Vtbl {{", t.name());
        _ = writeln!(
            code,
            "    {vis}QueryInterface: unsafe extern \"system\" fn(This: *mut c_void, riid: *const GUID, ppvObject: *mut *mut c_void) -> HRESULT,"
        );
        _ = writeln!(
            code,
            "    {vis}AddRef: unsafe extern \"system\" fn(This: *mut c_void) -> HRESULT,"
        );
        _ = writeln!(
            code,
            "    {vis}Release: unsafe extern \"system\" fn(This: *mut c_void) -> HRESULT,"
        );
        for entry in &ancestor_entries {
            _ = writeln!(code, "    {vis}{entry}");
        }
        for entry in interface_vtbl_entries.get(t.name()).into_iter().flatten() {
            _ = writeln!(code, "    {vis}{entry}");
        }
        _ = writeln!(code, "}}");
        _ = writeln!(code);
    }

    let out_path = PathBuf::from(out_dir).join("webview2_bindings.rs");
    std::fs::write(&out_path, code).expect("Failed to write webview2_bindings.rs");
}

fn collect_ancestor_vtbl_entries(
    name: &str,
    parent_map: &std::collections::HashMap<String, String>,
    vtbl_entries: &std::collections::HashMap<String, Vec<String>>,
) -> Vec<String> {
    if let Some(parent) = parent_map.get(name) {
        let mut entries = collect_ancestor_vtbl_entries(parent, parent_map, vtbl_entries);
        entries.extend(
            vtbl_entries
                .get(parent.as_str())
                .cloned()
                .unwrap_or_default(),
        );
        entries
    } else {
        vec![]
    }
}

fn collect_ancestor_impl_methods(
    name: &str,
    parent_map: &std::collections::HashMap<String, String>,
    impl_methods: &std::collections::HashMap<String, Vec<ImplMethodData>>,
) -> Vec<ImplMethodData> {
    if let Some(parent) = parent_map.get(name) {
        let mut entries = collect_ancestor_impl_methods(parent, parent_map, impl_methods);
        entries.extend(
            impl_methods
                .get(parent.as_str())
                .cloned()
                .unwrap_or_default(),
        );
        entries
    } else {
        vec![]
    }
}

fn map_type(
    ty: &windows_metadata::Type,
    enum_types: &std::collections::HashMap<String, String>,
    struct_types: &std::collections::HashSet<String>,
) -> String {
    use windows_metadata::Type;
    match ty {
        Type::Void => "()".to_string(),
        Type::I8 => "i8".to_string(),
        Type::U8 => "u8".to_string(),
        Type::I16 => "i16".to_string(),
        Type::U16 => "u16".to_string(),
        Type::I32 => "i32".to_string(),
        Type::U32 => "u32".to_string(),
        Type::I64 => "i64".to_string(),
        Type::U64 => "u64".to_string(),
        Type::F32 => "f32".to_string(),
        Type::F64 => "f64".to_string(),
        Type::ISize => "isize".to_string(),
        Type::USize => "usize".to_string(),
        Type::Name(tn) => map_named_type(tn, enum_types, struct_types),
        Type::PtrMut(inner, depth) => {
            let inner_mapped = map_type(inner, enum_types, struct_types);
            format!("{}{}", "*mut ".repeat(*depth), inner_mapped)
        }
        Type::PtrConst(inner, depth) => {
            let inner_mapped = map_type(inner, enum_types, struct_types);
            format!("{}{}", "*const ".repeat(*depth), inner_mapped)
        }
        _ => "*mut c_void".to_string(),
    }
}

fn map_named_type(
    tn: &windows_metadata::TypeName,
    enum_types: &std::collections::HashMap<String, String>,
    struct_types: &std::collections::HashSet<String>,
) -> String {
    match (tn.namespace.as_str(), tn.name.as_str()) {
        // Foundation types (value types passed by value)
        ("Windows.Win32.Foundation", "HRESULT") => "HRESULT".to_string(),
        ("Windows.Win32.Foundation", "HWND") => "HWND".to_string(),
        ("Windows.Win32.Foundation", "BOOL") => "BOOL".to_string(),
        ("Windows.Win32.Foundation", "RECT") => "RECT".to_string(),
        ("Windows.Win32.Foundation", "POINT") => "POINT".to_string(),
        ("Windows.Win32.Foundation", "HANDLE") => "HANDLE".to_string(),
        // PWSTR used bare in method params = input (const) mut wide string pointer
        ("Windows.Win32.Foundation", "PWSTR") => "*mut w_char".to_string(),
        // External COM types — interfaces always passed by pointer
        ("Windows.Win32.System.Com", "IStream") => "*mut IStream".to_string(),
        ("Windows.Win32.System.Com", "IUnknown") => "*mut c_void".to_string(),
        ("Windows.Win32.System.Com", "IDataObject") => "*mut c_void".to_string(),
        ("Windows.Win32.System.Variant", "VARIANT") => "*mut c_void".to_string(),
        // EventRegistrationToken is a value-type struct defined in generated code
        ("Windows.Win32.System.WinRT", "EventRegistrationToken") => {
            "EventRegistrationToken".to_string()
        }
        ("Windows.Win32.UI.WindowsAndMessaging", "HCURSOR") => "HCURSOR".to_string(),
        // WebView2 namespace types
        ("Microsoft.Web.WebView2.Win32", name) => {
            if let Some(underlying) = enum_types.get(name) {
                // Enum → underlying integer type
                underlying.clone()
            } else if struct_types.contains(name) {
                // Struct value type → pass by value
                name.to_string()
            } else {
                // Interface → always passed by pointer
                format!("*mut {name}")
            }
        }
        _ => "*mut c_void".to_string(),
    }
}
