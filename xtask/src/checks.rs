/*
 * Copyright (c) 2024-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::process::Command;
use std::sync::LazyLock;
use std::{env, fs};

use anyhow::{Context, Result, bail};
use regex::Regex;
use serde_json::Value;

use crate::metadata::{
    add_excludes, features_without_swap, package_for_directory, platform_excludes,
};
use crate::process::{capture, npm_program, npx_program, run, run_named};
use crate::utils::{collect_files, relative_slash};
use crate::{Os, Xtask};

pub(crate) const BACKEND_SWAP_PAIRS: [(&str, &str); 2] =
    [("native-tls", "vendored"), ("bsqlite", "bundled")];

static COPYRIGHT_HEADER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Copyright \(c\) 20[0-9]{2}(-20[0-9]{2})? \w+").expect("valid regex")
});

static UNSAFE_BLOCK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"unsafe\s*\{").expect("valid regex"));

impl Xtask {
    pub(crate) fn check(&self) -> Result<()> {
        self.check_copyright()?;
        self.check_formatting()?;
        self.check_docker()?;
        self.check_rust()?;
        self.check_rust_deps()?;
        self.check_e2e()
    }

    pub(crate) fn check_shared(&self) -> Result<()> {
        self.check_copyright()?;
        self.check_formatting()?;
        self.check_docker()?;
        self.check_rust_deps()
    }

    pub(crate) fn check_copyright(&self) -> Result<()> {
        println!("Checking copyright headers...");
        let extensions = ["rs", "html", "css", "js", "jsx", "ts", "tsx", "cc", "hh"];
        let mut bad = Vec::new();
        for file in self.source_files(&extensions)? {
            let relative = relative_slash(&self.root, &file);
            if relative.starts_with("bin/bob/examples/")
                || relative.ends_with(".min.js")
                || (relative.starts_with("bin/ccontinue/tests/")
                    && file.extension().is_some_and(|ext| ext == "cc"))
            {
                continue;
            }
            let contents = fs::read_to_string(&file)
                .with_context(|| format!("failed to read {}", file.display()))?;
            if !COPYRIGHT_HEADER.is_match(&contents) {
                bad.push(relative);
            }
        }
        if !bad.is_empty() {
            for file in bad {
                eprintln!("Bad copyright header in: {file}");
            }
            bail!("copyright header check failed");
        }
        Ok(())
    }

    pub(crate) fn check_formatting(&self) -> Result<()> {
        self.ensure_npm_deps()?;
        println!("Checking Prettier formatting...");
        let extensions = [
            "md", "json", "yml", "yaml", "html", "css", "js", "jsx", "ts", "tsx",
        ];
        let files = self
            .source_files(&extensions)?
            .into_iter()
            .filter(|file| {
                let relative = relative_slash(&self.root, file);
                !relative.starts_with(".vscode/")
                    && !relative.contains("/playwright/")
                    && !relative.contains("/test-results/")
                    && !relative.ends_with(".min.js")
            })
            .collect::<Vec<_>>();
        for files in files.chunks(100) {
            let mut command = Command::new(npx_program(self.os));
            command.args(["--no-install", "prettier", "--check"]);
            command.args(files);
            run_named(
                &mut command,
                &format!(
                    "{} prettier --check ({} files)",
                    npx_program(self.os),
                    files.len()
                ),
            )?;
        }

        println!("Checking clang-format formatting...");
        let mut files = Vec::new();
        collect_files(&self.root.join("bin/bob/examples"), &mut files)?;
        files.retain(|file| {
            !relative_slash(&self.root, file).contains("/target/")
                && file
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| {
                        matches!(ext, "c" | "h" | "cpp" | "hpp" | "m" | "mm" | "java")
                    })
        });
        files.sort();
        for files in files.chunks(100) {
            let mut command = Command::new("clang-format");
            command.args(["--dry-run", "--Werror"]);
            command.args(files);
            run_named(
                &mut command,
                &format!("clang-format --dry-run --Werror ({} files)", files.len()),
            )?;
        }
        Ok(())
    }

    pub(crate) fn check_docker(&self) -> Result<()> {
        println!("Checking Dockerfiles...");
        let mut files = self.source_files(&[])?;
        files.retain(|file| {
            let name = file
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default();
            name == "Dockerfile" || name.ends_with(".Dockerfile")
        });
        for file in files {
            run(Command::new("hadolint").arg(file))?;
        }
        Ok(())
    }

    pub(crate) fn check_rust(&self) -> Result<()> {
        let metadata = self.cargo_metadata()?;
        let excludes = platform_excludes(&metadata, self.os);

        println!("Checking Rust formatting...");
        run(Command::new("cargo").args(["+nightly", "fmt", "--", "--check"]))?;

        println!("Linting Rust code...");
        let mut command = Command::new("cargo");
        command.args(["clippy", "--workspace"]);
        add_excludes(&mut command, &excludes);
        command.args([
            "--locked",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
            "-W",
            "clippy::uninlined_format_args",
        ]);
        run(&mut command)?;

        println!("Running Rust tests...");
        let mut command = Command::new("cargo");
        command.args(["test", "--doc", "--all-features", "--locked", "--workspace"]);
        add_excludes(&mut command, &excludes);
        run(&mut command)?;

        let mut command = Command::new("cargo");
        command.args([
            "nextest",
            "run",
            "--all-features",
            "--locked",
            "--config-file",
            "nextest.toml",
        ]);
        if env::var_os("CI").is_some_and(|value| !value.is_empty()) {
            command.args(["--profile", "ci"]);
        }
        command.arg("--workspace");
        add_excludes(&mut command, &excludes);
        run(&mut command)?;

        for (package, feature) in BACKEND_SWAP_PAIRS {
            if excludes.contains(package) {
                continue;
            }
            println!("Running Rust tests for {package} without {feature} feature...");
            let feature_args = features_without_swap(&metadata, package, feature)?;
            let mut command = Command::new("cargo");
            command.args(["test", "--doc", "--locked", "-p", package]);
            command.args(&feature_args);
            run(&mut command)?;

            let mut command = Command::new("cargo");
            command.args([
                "nextest",
                "run",
                "--locked",
                "--config-file",
                "nextest.toml",
            ]);
            if env::var_os("CI").is_some_and(|value| !value.is_empty()) {
                command.args(["--profile", "ci"]);
            }
            command.args(["-p", package]);
            command.args(&feature_args);
            run(&mut command)?;
        }

        if self.os != Os::Windows {
            self.check_address_sanitizer(&metadata)?;
        }
        Ok(())
    }

    fn check_address_sanitizer(&self, metadata: &Value) -> Result<()> {
        println!("Running Rust tests with address sanitizer on unsafe libs...");
        let output = capture(Command::new("rustc").args(["+nightly", "-vV"]))?;
        let rustc_info = String::from_utf8(output.stdout)?;
        let target = rustc_info
            .lines()
            .find_map(|line| line.strip_prefix("host: "))
            .context("rustc did not report its host target")?;

        for entry in fs::read_dir(self.root.join("lib"))? {
            let crate_dir = entry?.path();
            if !crate_dir.is_dir() || crate_dir.ends_with("bwebview") {
                continue;
            }
            let mut rust_files = Vec::new();
            collect_files(&crate_dir, &mut rust_files)?;
            rust_files.retain(|file| file.extension().is_some_and(|ext| ext == "rs"));
            let has_unsafe =
                rust_files
                    .into_iter()
                    .try_fold(false, |found, file| -> Result<bool> {
                        Ok(found || contains_unsafe_block(&fs::read_to_string(file)?))
                    })?;
            if !has_unsafe {
                continue;
            }
            let package = package_for_directory(metadata, &crate_dir, &self.root)?;
            println!("Testing {package} with address sanitizer...");
            let swap = BACKEND_SWAP_PAIRS
                .iter()
                .find_map(|(candidate, feature)| (*candidate == package).then_some(*feature));
            let feature_args = match swap {
                Some(feature) => features_without_swap(metadata, package, feature)?,
                None => vec!["--all-features".to_owned()],
            };
            let mut command = Command::new("cargo");
            command.env("RUSTFLAGS", "-Zsanitizer=address");
            command.args([
                "+nightly", "test", "-p", package, "--lib", "--tests", "--locked",
            ]);
            command.args(feature_args);
            command.args(["--target", target, "-Zbuild-std"]);
            run(&mut command)?;
        }
        Ok(())
    }

    pub(crate) fn check_rust_deps(&self) -> Result<()> {
        println!("Checking Rust dependencies...");
        run(Command::new("cargo").args(["deny", "check", "--hide-inclusion-graph"]))
    }

    pub(crate) fn check_e2e(&self) -> Result<()> {
        println!("Running end-to-end tests...");
        self.ensure_npm_deps()?;
        run(Command::new("cargo").args(["build", "-p", "plaatnotes", "--locked"]))?;
        run(Command::new(npx_program(self.os)).args([
            "--no-install",
            "--workspace",
            "plaatnotes",
            "playwright",
            "install",
            "--with-deps",
        ]))?;
        run(Command::new(npm_program(self.os)).args(["test", "--workspace", "plaatnotes"]))
    }

    pub(crate) fn coverage(&self) -> Result<()> {
        let metadata = self.cargo_metadata()?;
        let excludes = platform_excludes(&metadata, self.os);
        let mut command = Command::new("cargo");
        command.args([
            "llvm-cov",
            "nextest",
            "--all-features",
            "--locked",
            "--no-fail-fast",
            "--retries",
            "2",
            "--workspace",
        ]);
        add_excludes(&mut command, &excludes);
        run(&mut command)
    }
}

fn contains_unsafe_block(contents: &str) -> bool {
    UNSAFE_BLOCK.is_match(contents)
}
