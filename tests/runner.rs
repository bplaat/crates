use std::fs;
use std::path::Path;
use std::process::Command;

const ENTITLEMENTS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.get-task-allow</key>
    <true/>
</dict>
</plist>
"#;

fn get_entitlements_path() -> String {
    let path = std::env::temp_dir().join("ccc_test.entitlements");
    fs::write(&path, ENTITLEMENTS_XML).expect("write entitlements");
    path.to_str().unwrap().to_owned()
}

fn parse_test_meta(filepath: &str) -> (i32, String) {
    let content = fs::read_to_string(filepath).expect("read test file");
    let mut expected_exit = 0i32;
    let mut out_lines: Vec<String> = Vec::new();
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("// EXIT: ") {
            expected_exit = rest.trim().parse().unwrap_or(0);
            continue;
        }
        if let Some(rest) = line.strip_prefix("// OUT: ") {
            out_lines.push(rest.to_owned());
            continue;
        }
        if !line.is_empty() && !line.starts_with("//") {
            break;
        }
    }
    let expected_stdout = if out_lines.is_empty() {
        String::new()
    } else {
        out_lines.join("\n") + "\n"
    };
    (expected_exit, expected_stdout)
}

fn build_test(test_file: &str) -> Result<String, String> {
    let ccc_bin = env!("CARGO_BIN_EXE_ccc");
    let exe_path = test_file.trim_end_matches(".cc").to_owned();
    let result = Command::new(ccc_bin)
        .arg(test_file)
        .output()
        .map_err(|e| format!("failed to run ccc: {e}"))?;

    if !result.status.success() || !Path::new(&exe_path).exists() {
        let stderr = String::from_utf8_lossy(&result.stderr).trim().to_owned();
        let stdout = String::from_utf8_lossy(&result.stdout).trim().to_owned();
        return Err(if !stderr.is_empty() {
            stderr
        } else if !stdout.is_empty() {
            stdout
        } else {
            "unknown build error".to_owned()
        });
    }

    // On macOS codesign with get-task-allow so leaks can attach
    if cfg!(target_os = "macos") {
        let _ = Command::new("codesign")
            .args([
                "--force",
                "--sign",
                "-",
                "--entitlements",
                &get_entitlements_path(),
                &exe_path,
            ])
            .output();
    }

    Ok(exe_path)
}

fn run_normal(exe_path: &str, expected_exit: i32, expected_stdout: &str) -> Result<(), String> {
    let result = Command::new(exe_path)
        .output()
        .map_err(|e| format!("failed to run {exe_path}: {e}"))?;
    let actual_exit = result.status.code().unwrap_or(-1);
    if actual_exit != expected_exit {
        return Err(format!(
            "exit code {} (expected {})",
            actual_exit, expected_exit
        ));
    }
    let actual_stdout = String::from_utf8_lossy(&result.stdout).into_owned();
    if actual_stdout != expected_stdout {
        let exp_repr = format!("{:?}", &expected_stdout[..expected_stdout.len().min(300)]);
        let got_repr = format!("{:?}", &actual_stdout[..actual_stdout.len().min(300)]);
        return Err(format!(
            "stdout mismatch\n    expected: {}\n    got:      {}",
            exp_repr, got_repr
        ));
    }
    Ok(())
}

fn run_leaks(exe_path: &str) -> Result<(), String> {
    if cfg!(target_os = "macos") {
        let result = Command::new("leaks")
            .args(["--atExit", "--", exe_path])
            .output()
            .map_err(|e| format!("leaks failed: {e}"))?;
        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        if combined.contains("0 leaks for 0 total leaked bytes") {
            return Ok(());
        }
        if !result.status.success() {
            let tail: String = combined
                .chars()
                .rev()
                .take(800)
                .collect::<String>()
                .chars()
                .rev()
                .collect();
            return Err(format!("leaks detected\n{}", tail));
        }
        Ok(())
    } else if cfg!(target_os = "linux") {
        let result = Command::new("valgrind")
            .args([
                "--leak-check=full",
                "--show-leak-kinds=all",
                "--track-origins=yes",
                "--error-exitcode=1",
                exe_path,
            ])
            .output()
            .map_err(|e| format!("valgrind failed: {e}"))?;
        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            let tail: String = stderr
                .chars()
                .rev()
                .take(800)
                .collect::<String>()
                .chars()
                .rev()
                .collect();
            return Err(format!("valgrind errors\n{}", tail));
        }
        Ok(())
    } else {
        Ok(()) // unsupported platform: skip
    }
}

fn run_test(test_file: &str) {
    let (expected_exit, expected_stdout) = parse_test_meta(test_file);

    let exe_path = match build_test(test_file) {
        Ok(p) => p,
        Err(e) => panic!("build error: {}", e),
    };

    let run_result = run_normal(&exe_path, expected_exit, &expected_stdout);
    let leaks_result = run_leaks(&exe_path);

    // Clean up binary
    let _ = fs::remove_file(&exe_path);

    if let Err(e) = run_result {
        panic!("{}", e);
    }
    if let Err(e) = leaks_result {
        panic!("[leaks] {}", e);
    }
}

include!(concat!(env!("OUT_DIR"), "/generated_tests.rs"));
