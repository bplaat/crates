/*
 * Copyright (c) 2024-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

use anyhow::{Context, Result};
use serde_json::Value;

use crate::metadata::{InstallableApp, packages, supports_os};
use crate::process::{npm_program, run};
use crate::utils::{
    collect_paths, copy_directory, copy_directory_contents, remove_directory_except, remove_path,
    remove_path_if_exists,
};
use crate::{Os, Xtask};

impl Xtask {
    pub(crate) fn clean(&self) -> Result<()> {
        if self.os == Os::Windows {
            // Windows does not allow `cargo clean` to delete the xtask executable
            // while it is running. Remove everything except that executable instead.
            println!("Cleaning Cargo artifacts...");
            let target = self.root.join("target");
            if target.exists() {
                let executable = env::current_exe().context("failed to locate xtask executable")?;
                remove_directory_except(&target, &executable)?;
            }
        } else {
            run(Command::new("cargo").arg("clean"))?;
        }
        let mut generated = Vec::new();
        collect_paths(&self.root, &mut generated, &|path, is_dir| {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default();
            if is_dir {
                (name == "target" && path != self.root.join("target"))
                    || matches!(
                        name,
                        "node_modules"
                            | "dist"
                            | "src-gen"
                            | "playwright"
                            | "playwright-report"
                            | "test-results"
                    )
            } else {
                name.contains(".db")
            }
        })?;
        for path in generated {
            remove_path(&path)?;
        }
        Ok(())
    }

    pub(crate) fn build_pages(&self) -> Result<()> {
        let pages = self.root.join("target/pages");
        fs::create_dir_all(&pages)?;
        fs::copy(self.root.join("index.html"), pages.join("index.html"))?;
        self.build_pages_baksteen(&pages)?;
        self.build_pages_plaatui(&pages)
    }

    fn build_pages_baksteen(&self, pages: &Path) -> Result<()> {
        let destination = pages.join("baksteen");
        fs::create_dir_all(&destination)?;
        copy_directory_contents(&self.root.join("bin/baksteen/public"), &destination)?;
        run(Command::new("cargo").args([
            "build",
            "--release",
            "-p",
            "baksteen",
            "--target",
            "wasm32-unknown-unknown",
        ]))?;
        run(Command::new("wasm-bindgen").args([
            "--target",
            "web",
            "--no-typescript",
            "--out-dir",
            destination
                .to_str()
                .context("pages path is not valid UTF-8")?,
            "--out-name",
            "baksteen",
            "target/wasm32-unknown-unknown/release/baksteen.wasm",
        ]))
    }

    fn build_pages_plaatui(&self, pages: &Path) -> Result<()> {
        self.ensure_npm_deps()?;
        run(Command::new(npm_program(self.os)).args([
            "run",
            "build-release",
            "--workspace",
            "plaatui-showcase",
        ]))?;
        let destination = pages.join("plaatui");
        fs::create_dir_all(&destination)?;
        copy_directory_contents(
            &self.root.join("npm-lib/plaatui/showcase/dist"),
            &destination,
        )
    }

    pub(crate) fn build_bundle(&self) -> Result<()> {
        run(Command::new("cargo").args(["install", "--path", "bin/cargo-bundle"]))?;
        for app in self.installable_apps()? {
            run(Command::new("cargo").args(["bundle", "--path", &format!("bin/{}", app.package)]))?;
        }
        Ok(())
    }

    pub(crate) fn install(&self) -> Result<()> {
        for package in ["bob", "ccontinue", "music-dl"] {
            run(Command::new("cargo").args([
                "install",
                "--force",
                "--path",
                &format!("bin/{package}"),
            ]))?;
        }

        match self.os {
            Os::Macos => {
                self.build_bundle()?;
                for app in self.installable_apps()? {
                    let bundle_directory = self.root.join(format!("target/bundle/{}", app.package));
                    let source = fs::read_dir(&bundle_directory)?
                        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
                        .find(|path| path.extension().is_some_and(|extension| extension == "app"))
                        .with_context(|| {
                            format!("no .app found in {}", bundle_directory.display())
                        })?;
                    let destination = Path::new("/Applications").join(
                        source
                            .file_name()
                            .context("application bundle has no file name")?,
                    );
                    remove_path_if_exists(&destination)?;
                    copy_directory(&source, &destination)?;
                }
            }
            Os::Windows => {
                let desktop =
                    PathBuf::from(env::var("USERPROFILE").context("USERPROFILE is not set")?)
                        .join("Desktop");
                for app in self.installable_apps()? {
                    run(Command::new("cargo").args(["build", "--release", "--bin", &app.package]))?;
                    fs::copy(
                        self.root
                            .join(format!("target/release/{}.exe", app.package)),
                        desktop.join(format!("{}.exe", app.name)),
                    )?;
                }
            }
            Os::Linux => {
                let home = PathBuf::from(env::var("HOME").context("HOME is not set")?);
                let bin = home.join(".local/bin");
                let applications = home.join(".local/share/applications");
                let icons = home.join(".local/share/icons");
                fs::create_dir_all(&bin)?;
                fs::create_dir_all(&applications)?;
                fs::create_dir_all(&icons)?;
                for app in self.installable_apps()? {
                    run(Command::new("cargo").args(["build", "--release", "--bin", &app.package]))?;
                    fs::copy(
                        self.root.join(format!("target/release/{}", app.package)),
                        bin.join(&app.name),
                    )?;
                    fs::copy(
                        self.root
                            .join(format!("bin/{}/meta/freedesktop/.desktop", app.package)),
                        applications.join(format!("{}.desktop", app.name)),
                    )?;
                    fs::copy(
                        self.root
                            .join(format!("bin/{}/docs/images/icon.svg", app.package)),
                        icons.join(format!("{}.svg", app.name)),
                    )?;
                }
            }
        }
        Ok(())
    }

    fn installable_apps(&self) -> Result<Vec<InstallableApp>> {
        let metadata = self.cargo_metadata()?;
        let mut apps = Vec::new();
        for package in packages(&metadata)? {
            if !supports_os(package, self.os) {
                continue;
            }
            let Some(bundle) = package.pointer("/metadata/bundle") else {
                continue;
            };
            let Some(identifier) = bundle.get("identifier").and_then(Value::as_str) else {
                continue;
            };
            let package_name = package
                .get("name")
                .and_then(Value::as_str)
                .context("Cargo package has no name")?;
            apps.push(InstallableApp {
                package: package_name.to_owned(),
                name: identifier
                    .rsplit('.')
                    .next()
                    .context("bundle identifier is empty")?
                    .to_owned(),
            });
        }
        Ok(apps)
    }
}
