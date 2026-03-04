// MARK: Args
use std::env;

pub struct Args {
    pub files: Vec<String>,
    pub output: Option<String>,
    pub include_paths: Vec<String>,
    pub flag_source: bool,
    pub flag_compile: bool,
    pub flag_run: bool,
    pub flag_run_leaks: bool,
}

pub fn parse_args() -> Args {
    let raw: Vec<String> = env::args().collect();
    let mut files = Vec::new();
    let mut output = None;
    let mut include_paths = Vec::new();
    let mut flag_source = false;
    let mut flag_compile = false;
    let mut flag_run = false;
    let mut flag_run_leaks = false;

    let mut i = 1;
    while i < raw.len() {
        match raw[i].as_str() {
            "-o" | "--output" => {
                i += 1;
                output = Some(raw[i].clone());
            }
            "-I" | "--include" => {
                i += 1;
                include_paths.push(raw[i].clone());
            }
            arg if arg.starts_with("-I") => {
                include_paths.push(arg[2..].to_owned());
            }
            "-S" | "--source" => flag_source = true,
            "-c" | "--compile" => flag_compile = true,
            "-r" | "--run" => flag_run = true,
            "-R" | "--run-leaks" => flag_run_leaks = true,
            arg if !arg.starts_with('-') => files.push(arg.to_owned()),
            _ => {
                eprintln!("Unknown argument: {}", raw[i]);
                std::process::exit(1);
            }
        }
        i += 1;
    }

    if files.is_empty() {
        eprintln!("Usage: ccc <file> [-o output] [-I include] [-S] [-c] [-r] [-R]");
        std::process::exit(1);
    }

    Args {
        files,
        output,
        include_paths,
        flag_source,
        flag_compile,
        flag_run,
        flag_run_leaks,
    }
}
