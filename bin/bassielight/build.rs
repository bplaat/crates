/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("README.md")]

fn main() {
    #[cfg(windows)]
    const NPM: &str = "npm.cmd";
    #[cfg(not(windows))]
    const NPM: &str = "npm";

    // Install npm packages if needed
    if !std::path::Path::new("web/node_modules").exists() {
        std::process::Command::new(NPM)
            .arg("ci")
            .arg("--prefer-offline")
            .current_dir("web")
            .output()
            .expect("Failed to run npm install");
    }

    // Invalidate build when web assets change
    fn print_rerun(dir: &std::path::Path) {
        for entry in std::fs::read_dir(dir).expect("Failed to read dir") {
            let path = entry.expect("Failed to read entry").path();
            if path.is_dir() {
                print_rerun(&path);
            } else {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }
    println!("cargo:rerun-if-changed=web/index.html");
    print_rerun(std::path::Path::new("web/src"));

    // Build frontend
    std::process::Command::new(NPM)
        .arg("run")
        .arg(if cfg!(debug_assertions) {
            "build-debug"
        } else {
            "build-release"
        })
        .current_dir("web")
        .output()
        .expect("Failed to run npm run build");

    // Copy built assets to OUT_DIR/web
    let out_dir = std::env::var("OUT_DIR").expect("Should be some");
    let dest_path = std::path::Path::new(&out_dir).join("web");
    if dest_path.exists() {
        std::fs::remove_dir_all(&dest_path).expect("Failed to remove old web dir");
    }
    fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dst)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            let dst_path = dst.join(entry.file_name());
            if ty.is_dir() {
                copy_dir_all(&entry.path(), &dst_path)?;
            } else {
                std::fs::copy(entry.path(), dst_path)?;
            }
        }
        Ok(())
    }
    copy_dir_all(std::path::Path::new("web/dist"), &dest_path)
        .expect("Failed to copy web/dist files to $OUT_DIR");

    // Compile Windows resources
    #[cfg(windows)]
    {
        let manifest = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
    <assemblyIdentity type="win32" name="BassieLight" version="{}.0" processorArchitecture="*"/>
    <description>BassieLight</description>

    <dependency>
        <dependentAssembly>
            <assemblyIdentity type="win32" name="Microsoft.Windows.Common-Controls" version="6.0.0.0" processorArchitecture="*" publicKeyToken="6595b64144ccf1df" language="*"/>
        </dependentAssembly>
    </dependency>

    <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
        <application>
            <!-- Windows 10 and Windows 11 -->
            <supportedOS Id="{{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}}"/>
        </application>
    </compatibility>

    <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
        <security>
            <requestedPrivileges>
                <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
            </requestedPrivileges>
        </security>
    </trustInfo>
</assembly>
"#,
            env!("CARGO_PKG_VERSION")
        );

        let mut res = winres::WindowsResource::new();
        res.set("ProductName", "BassieLight")
            .set("FileDescription", "BassieLight")
            .set("LegalCopyright", "Copyright © 2025 Bastiaan van der Plaat")
            .set_icon("meta/windows/icon.ico")
            .set_manifest(&manifest);
        res.compile().expect("Failed to compile Windows resources.");
    }
}
