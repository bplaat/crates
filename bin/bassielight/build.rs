/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("README.md")]

use std::path::Path;

use copy_dir::copy_dir;

fn main() {
    #[cfg(windows)]
    const NPM: &str = "npm.cmd";
    #[cfg(not(windows))]
    const NPM: &str = "npm";

    // Install npm packages if needed
    if !Path::new("web/node_modules").exists() {
        std::process::Command::new(NPM)
            .arg("ci")
            .arg("--prefer-offline")
            .current_dir("web")
            .output()
            .expect("Failed to run npm install");
    }

    // Invalidate build when web assets change
    fn print_rerun(dir: &Path) {
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
    print_rerun(Path::new("web/src"));

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
    copy_dir("web/dist", Path::new(&out_dir).join("web"))
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
