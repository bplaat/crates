/*
 * Copyright (c) 2024-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::process::{Command, Output};

use anyhow::{Context, Result, bail};

use crate::Os;

pub(crate) fn run(command: &mut Command) -> Result<()> {
    let label = display_command(command);
    run_named(command, &label)
}

pub(crate) fn run_named(command: &mut Command, label: &str) -> Result<()> {
    println!("$ {label}");
    let status = command
        .status()
        .with_context(|| format!("failed to start {label}"))?;
    if !status.success() {
        bail!("command failed with {status}: {label}");
    }
    Ok(())
}

pub(crate) fn capture(command: &mut Command) -> Result<Output> {
    let output = command
        .output()
        .with_context(|| format!("failed to start {}", display_command(command)))?;
    if !output.status.success() {
        bail!(
            "command failed with {}: {}\n{}",
            output.status,
            display_command(command),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(output)
}

fn display_command(command: &Command) -> String {
    std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(|argument| argument.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn npm_program(os: Os) -> &'static str {
    if os == Os::Windows { "npm.cmd" } else { "npm" }
}

pub(crate) fn npx_program(os: Os) -> &'static str {
    if os == Os::Windows { "npx.cmd" } else { "npx" }
}

// MARK: Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn program_names_depend_on_os() {
        assert_eq!(npm_program(Os::Linux), "npm");
        assert_eq!(npm_program(Os::Macos), "npm");
        assert_eq!(npm_program(Os::Windows), "npm.cmd");
        assert_eq!(npx_program(Os::Linux), "npx");
        assert_eq!(npx_program(Os::Windows), "npx.cmd");
    }

    #[test]
    fn display_command_joins_program_and_arguments() {
        let mut command = Command::new("cargo");
        command.args(["build", "--release"]);
        assert_eq!(display_command(&command), "cargo build --release");
    }

    #[cfg(unix)]
    #[test]
    fn run_reports_command_exit_status() {
        assert!(run(&mut Command::new("true")).is_ok());
        assert!(run(&mut Command::new("false")).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn capture_returns_stdout_on_success() -> Result<()> {
        let output = capture(Command::new("echo").arg("hello"))?;
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "hello");
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn capture_errors_on_failure_status() {
        assert!(capture(&mut Command::new("false")).is_err());
    }
}
