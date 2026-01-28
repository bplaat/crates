/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [winresource](https://crates.io/crates/winresource) crate

#![forbid(unsafe_code)]

use core::panic;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

use crate::version::MicrosoftVersion;

mod version;

/// Windows resource compiler
///
/// Supports msvc rc.exe, mingw windres and zig rc.
pub struct WindowsResource {
    icon_path: Option<PathBuf>,
    manifest: Option<String>,
    version_fields: HashMap<String, String>,
}

impl Default for WindowsResource {
    fn default() -> Self {
        let mut version_fields = HashMap::new();
        version_fields.insert(
            "FileVersion".to_string(),
            env::var("CARGO_PKG_VERSION").expect("CARGO_PKG_VERSION not set"),
        );
        version_fields.insert(
            "ProductVersion".to_string(),
            env::var("CARGO_PKG_VERSION").expect("CARGO_PKG_VERSION not set"),
        );
        version_fields.insert(
            "FileDescription".to_string(),
            env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME not set"),
        );
        version_fields.insert(
            "ProductName".to_string(),
            env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME not set"),
        );

        Self {
            icon_path: None,
            manifest: None,
            version_fields,
        }
    }
}

impl WindowsResource {
    /// Create a new Windows resource compiler
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a version field
    pub fn set(&mut self, key: impl AsRef<str>, value: impl AsRef<str>) -> &mut Self {
        self.version_fields
            .insert(key.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Set the icon file
    pub fn set_icon(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.icon_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set the manifest content
    pub fn set_manifest(&mut self, manifest: &str) -> &mut Self {
        self.manifest = Some(manifest.to_string());
        self
    }

    /// Compile the resources
    pub fn compile(&self) -> Result<(), String> {
        let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable not set");

        // Write manifest file
        if let Some(manifest) = &self.manifest {
            let manifest_path = Path::new(&out_dir).join("manifest.xml");
            if let Some(parent) = manifest_path.parent() {
                fs::create_dir_all(parent).unwrap_or_else(|_| {
                    panic!("failed to create output directory {}", parent.display())
                });
            }
            fs::write(&manifest_path, manifest.as_bytes()).unwrap_or_else(|_| {
                panic!("failed to write manifest to {}", manifest_path.display())
            });
        }

        // Write resource.rc file
        let mut rc_content = "#pragma code_page(65001)\r\n\r\n".to_string();

        if let Some(icon_path) = &self.icon_path {
            rc_content.push_str(&format!(
                "1 ICON \"{}\"\r\n\r\n",
                escape_string(&icon_path.display().to_string())
            ));
        }

        if self.manifest.is_some() {
            rc_content.push_str(&format!(
                "1 24 \"{}\"\r\n\r\n",
                escape_string(
                    &Path::new(&out_dir)
                        .join("manifest.xml")
                        .display()
                        .to_string()
                )
            ));
        }

        let version = semver::Version::parse(
            &env::var("CARGO_PKG_VERSION").expect("CARGO_PKG_VERSION not set"),
        )
        .expect("Can't parse version semver");
        rc_content.push_str(&format!(
            "1 VERSIONINFO\r\n\
        FILEVERSION {maj},{min},{pat},0\r\n\
        PRODUCTVERSION {maj},{min},{pat},0\r\n\
        FILEOS 0x00040004\r\n\
        FILETYPE 1\r\n\
        FILESUBTYPE 0\r\n\
        FILEFLAGSMASK 0x3F\r\n\
        FILEFLAGS 0\r\n\
        BEGIN\r\n\
          BLOCK \"StringFileInfo\"\r\n\
          BEGIN\r\n\
            BLOCK \"040904B0\"\r\n\
            BEGIN\r\n",
            maj = version.major,
            min = version.minor,
            pat = version.patch
        ));
        for (k, v) in &self.version_fields {
            rc_content.push_str(&format!(
                "VALUE \"{k}\", \"{val}\"\n",
                k = escape_string(k),
                val = escape_string(v)
            ));
        }
        rc_content.push_str(
            "END\r\n\
          END\r\n\
          BLOCK \"VarFileInfo\"\r\n\
          BEGIN\r\n\
            VALUE \"Translation\", 0x0409, 0x04B0\r\n\
          END\r\n\
        END\n",
        );

        let rc_path = Path::new(&out_dir).join("resource.rc");
        fs::write(&rc_path, rc_content)
            .unwrap_or_else(|_| panic!("failed to write resource.rc to {}", rc_path.display()));

        // Compile resource.rc
        if env::var("RUSTC_LINKER").unwrap_or_default().contains("zig") {
            let status = Command::new("zig")
                .arg("rc")
                .arg("/fo")
                .arg(Path::new(&out_dir).join("resource.lib"))
                .arg(&rc_path)
                .status()
                .map_err(|e| format!("failed to execute rc.exe: {e}"))?;
            if !status.success() {
                return Err(format!(
                    "zig rc failed with exit code: {}",
                    status.code().unwrap_or(-1)
                ));
            }
            println!("cargo:rustc-link-search=native={out_dir}");
            println!("cargo:rustc-link-lib=static=resource");
            return Ok(());
        }

        match env::var("CARGO_CFG_TARGET_ENV")
            .unwrap_or_default()
            .as_str()
        {
            "msvc" => {
                let status = Command::new(find_rc_exe().expect("Can't find rc.exe"))
                    .arg("/fo")
                    .arg(Path::new(&out_dir).join("resource.lib"))
                    .arg(&rc_path)
                    .status()
                    .map_err(|e| format!("failed to execute rc.exe: {e}"))?;
                if !status.success() {
                    return Err(format!(
                        "rc.exe failed with exit code: {}",
                        status.code().unwrap_or(-1)
                    ));
                }
                println!("cargo:rustc-link-search=native={out_dir}");
                println!("cargo:rustc-link-lib=static=resource");
                Ok(())
            }
            "gnu" => {
                let object_path = Path::new(&out_dir).join("resource.o");
                let tools = [
                    "windres",
                    "x86_64-w64-mingw32-windres",
                    "i686-w64-mingw32-windres",
                ];
                let mut last_error = None;
                for tool in &tools {
                    let status = Command::new(tool)
                        .arg(&rc_path)
                        .arg("-O")
                        .arg("coff")
                        .arg("-o")
                        .arg(&object_path)
                        .status();
                    match status {
                        Ok(s) if s.success() => {
                            last_error = None;
                            break;
                        }
                        Ok(s) => {
                            last_error = Some(format!(
                                "{} failed with exit code: {}",
                                tool,
                                s.code().unwrap_or(-1)
                            ));
                        }
                        Err(e) => {
                            last_error = Some(format!("failed to execute {tool}: {e}"));
                        }
                    }
                }
                if let Some(err) = last_error {
                    return Err(err);
                }
                println!("cargo:rustc-link-arg={}", object_path.display());
                Ok(())
            }
            other => Err(format!("unsupported target environment: {other}")),
        }
    }
}

fn find_rc_exe() -> Option<PathBuf> {
    let kit_root = Path::new(r"C:\Program Files (x86)\Windows Kits\10\bin");
    if !kit_root.exists() {
        return None;
    }

    let arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "x86"
    };

    let mut best_version: Option<MicrosoftVersion> = None;
    let mut best_path: Option<PathBuf> = None;
    if let Ok(entries) = fs::read_dir(kit_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            if let Some(version_str) = path.file_name().and_then(|s| s.to_str())
                && let Ok(version) = MicrosoftVersion::parse(version_str)
            {
                let rc_path = path.join(arch).join("rc.exe");
                if rc_path.exists()
                    && (best_version.is_none()
                        || &version > best_version.as_ref().expect("Should be some"))
                {
                    best_version = Some(version.clone());
                    best_path = Some(rc_path);
                }
            }
        }
    }
    best_path
}

fn escape_string(string: &str) -> String {
    let mut escaped = String::new();
    for chr in string.chars() {
        match chr {
            '"' => escaped.push_str("\"\""),
            '\'' => escaped.push_str("\\'"),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\t' => escaped.push_str("\\t"),
            '\r' => escaped.push_str("\\r"),
            _ => escaped.push(chr),
        };
    }
    escaped
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn string_escaping() {
        assert_eq!(&escape_string(""), "");
        assert_eq!(&escape_string("foo"), "foo");
        assert_eq!(&escape_string(r#""Hello""#), r#"""Hello"""#);
        assert_eq!(
            &escape_string(r"C:\Program Files\Foobar"),
            r"C:\\Program Files\\Foobar"
        );
    }
}
