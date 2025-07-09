/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A [navidrome.plaatsoft.nl](https://navidrome.plaatsoft.nl/) webview wrapper

fn main() {
    // Generate resources for macOS bundle
    #[cfg(target_os = "macos")]
    {
        let target_dir = "../../target/Navidrome"; // FIXME: Find way to not hardcode this path
        std::fs::create_dir_all(target_dir).expect("Failed to create target directory");

        // Create icon.icns
        std::process::Command::new("iconutil")
            .args([
                "-c",
                "icns",
                "meta/macos/icon.iconset",
                "-o",
                &format!("{}/icon.icns", target_dir),
            ])
            .output()
            .expect("Failed to create icon.icns");

        // Generate Info.plist
        let info_plist = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundlePackageType</key>
	<string>APPL</string>
	<key>CFBundleName</key>
	<string>Navidrome</string>
	<key>CFBundleDisplayName</key>
	<string>Navidrome</string>
	<key>CFBundleIdentifier</key>
	<string>nl.bplaat.Navidrome</string>
	<key>CFBundleVersion</key>
	<string>{}</string>
	<key>CFBundleShortVersionString</key>
	<string>{}</string>
	<key>CFBundleExecutable</key>
	<string>Navidrome</string>
	<key>LSMinimumSystemVersion</key>
	<string>11.0</string>
	<key>CFBundleIconFile</key>
	<string>icon</string>
	<key>NSHumanReadableCopyright</key>
	<string>Copyright © 2025 Bastiaan van der Plaat</string>
</dict>
</plist>"#,
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_VERSION")
        );
        std::fs::write(format!("{}/Info.plist", target_dir), info_plist)
            .expect("Failed to write Info.plist");
    }

    // Compile Windows resources
    #[cfg(windows)]
    {
        let manifest = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
    <assemblyIdentity type="win32" name="Navidrome" version="{}.0" processorArchitecture="*"/>
    <description>Navidrome</description>

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
        res.set("ProductName", "Navidrome")
            .set("FileDescription", "Navidrome")
            .set("LegalCopyright", "Copyright © 2025 Bastiaan van der Plaat")
            .set_icon("meta/windows/icon.ico")
            .set_manifest(&manifest);
        res.compile().expect("Failed to compile Windows resources.");
    }
}
