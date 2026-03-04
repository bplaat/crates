#!/usr/bin/env python3
"""cContinue test runner - compiles and runs all tests/*.cc, checks stdout/exit code,
then re-runs each under leaks (macOS) or valgrind (Linux) to check for memory leaks."""

import os
import platform
import re
import subprocess
import sys
import tempfile

# Entitlements plist that grants get-task-allow so leaks/lldb can attach on macOS
_ENTITLEMENTS_XML = """\
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" \
"http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.get-task-allow</key>
    <true/>
</dict>
</plist>
"""

# Path to the shared entitlements file (written once per run, reused for all tests)
_entitlements_path: str | None = None


def _get_entitlements_path() -> str:
    global _entitlements_path
    if _entitlements_path is None:
        fd, path = tempfile.mkstemp(suffix=".entitlements")
        os.write(fd, _ENTITLEMENTS_XML.encode())
        os.close(fd)
        _entitlements_path = path
    return _entitlements_path


def parse_test_meta(filepath: str) -> tuple[int, str]:
    """Parse // EXIT: n and // OUT: line comments from a test file header."""
    expected_exit = 0
    out_lines: list[str] = []
    with open(filepath, encoding="utf-8") as f:
        for line in f:
            line = line.rstrip("\n")
            m = re.match(r"^// EXIT: (\d+)$", line)
            if m:
                expected_exit = int(m.group(1))
                continue
            m = re.match(r"^// OUT: (.*)$", line)
            if m:
                out_lines.append(m.group(1))
                continue
            if line and not line.startswith("//"):
                break
    return expected_exit, ("\n".join(out_lines) + "\n" if out_lines else "")


def build_test(ccc_path: str, test_file: str) -> tuple[str | None, str | None]:
    """Transpile, compile and link test_file. Returns (exe_path, error_msg)."""
    # Let ccc.py derive the exe name automatically (removes .cc suffix)
    exe_path = test_file[: -len(".cc")]
    result = subprocess.run(
        [sys.executable, ccc_path, test_file],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0 or not os.path.exists(exe_path):
        return None, (result.stderr.strip() or result.stdout.strip() or "unknown build error")

    # On macOS codesign with get-task-allow so leaks can attach to the process
    if platform.system() == "Darwin":
        subprocess.run(
            ["codesign", "--force", "--sign", "-", "--entitlements", _get_entitlements_path(), exe_path],
            capture_output=True,
        )

    return exe_path, None


def run_normal(exe_path: str, expected_exit: int, expected_stdout: str) -> tuple[bool, str | None]:
    """Run exe normally; verify exit code and stdout."""
    result = subprocess.run([exe_path], capture_output=True, text=True)
    if result.returncode != expected_exit:
        return False, f"exit code {result.returncode} (expected {expected_exit})"
    if result.stdout != expected_stdout:
        exp_repr = repr(expected_stdout[:300])
        got_repr = repr(result.stdout[:300])
        return False, f"stdout mismatch\n    expected: {exp_repr}\n    got:      {got_repr}"
    return True, None


def run_leaks(exe_path: str) -> tuple[bool, str | None]:
    """Run exe under leak checker. Returns (passed, error_msg_or_None)."""
    system = platform.system()
    if system == "Darwin":
        result = subprocess.run(
            ["leaks", "--atExit", "--", exe_path],
            capture_output=True,
            text=True,
        )
        combined = result.stdout + result.stderr
        if "0 leaks for 0 total leaked bytes" in combined:
            return True, None
        if result.returncode != 0:
            return False, f"leaks detected\n{combined[-800:]}"
        return True, None
    elif system == "Linux":
        result = subprocess.run(
            [
                "valgrind",
                "--leak-check=full",
                "--show-leak-kinds=all",
                "--track-origins=yes",
                "--error-exitcode=1",
                exe_path,
            ],
            capture_output=True,
            text=True,
        )
        if result.returncode != 0:
            return False, f"valgrind errors\n{result.stderr[-800:]}"
        return True, None
    else:
        return True, "skipped (unsupported platform)"


def main() -> None:
    script_dir = os.path.dirname(os.path.abspath(__file__))
    ccc_path = os.path.join(script_dir, "ccc.py")
    tests_dir = os.path.join(script_dir, "tests")

    test_files = sorted(os.path.join(tests_dir, f) for f in os.listdir(tests_dir) if f.endswith(".cc"))
    if not test_files:
        print("No test files found in tests/")
        sys.exit(1)

    total = len(test_files)
    passed = 0
    failed: list[str] = []

    print(f"Running {total} test(s)...")
    print()

    for test_file in test_files:
        name = os.path.basename(test_file)
        expected_exit, expected_stdout = parse_test_meta(test_file)

        # Build
        exe_path, build_err = build_test(ccc_path, test_file)
        if exe_path is None:
            print(f"  FAIL  {name}")
            print(f"        build error: {build_err}")
            failed.append(name)
            continue

        test_ok = True
        try:
            # Normal run
            ok, msg = run_normal(exe_path, expected_exit, expected_stdout)
            if ok:
                print(f"  pass  {name}")
            else:
                print(f"  FAIL  {name}")
                print(f"        {msg}")
                test_ok = False

            # Leaks run
            ok, msg = run_leaks(exe_path)
            if ok:
                extra = f" ({msg})" if msg else ""
                print(f"  pass  {name} [leaks{extra}]")
            else:
                print(f"  FAIL  {name} [leaks]")
                print(f"        {msg}")
                test_ok = False
        finally:
            try:
                os.unlink(exe_path)
            except OSError:
                pass

        if test_ok:
            passed += 1
        else:
            failed.append(name)

    print()
    print(f"Results: {passed}/{total} passed")
    if _entitlements_path and os.path.exists(_entitlements_path):
        os.unlink(_entitlements_path)
    if failed:
        print(f"Failed:  {', '.join(failed)}")
        sys.exit(1)
    sys.exit(0)


if __name__ == "__main__":
    main()
