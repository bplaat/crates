use std::fs;
use std::path::Path;

fn main() {
    let tests_dir = Path::new("tests");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("generated_tests.rs");

    let mut test_fns = String::new();
    let mut entries: Vec<_> = fs::read_dir(tests_dir)
        .expect("tests/ dir should exist")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "cc").unwrap_or(false))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in &entries {
        let path = entry.path();
        let stem = path.file_stem().unwrap().to_str().unwrap();
        // Sanitize name: digits and letters only (replace non-alnum with _)
        let fn_name: String = stem
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect();
        let path_str = path.to_str().unwrap();
        test_fns.push_str(&format!(
            "#[test]\nfn test_{fn_name}() {{\n    run_test({path_str:?});\n}}\n\n"
        ));
        println!("cargo:rerun-if-changed={path_str}");
    }

    fs::write(&dest, test_fns).unwrap();
    println!("cargo:rerun-if-changed=tests/");
}
