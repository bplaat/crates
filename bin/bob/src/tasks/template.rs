/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::path::Path;
use std::{env, fs};

use crate::Bobje;
use crate::executor::Executor;
use crate::utils::write_file_when_different;

// MARK: Template tasks
pub(crate) fn detect_template(source_files: &[String]) -> bool {
    source_files.iter().any(|path| path.ends_with(".in"))
}

pub(crate) fn process_templates(bobje: &mut Bobje, _executor: &mut Executor) {
    let regex = regex::Regex::new(r"@([A-Z0-9_]+)@").expect("Can't compile regex");
    let template_paths = bobje
        .source_files
        .iter()
        .filter(|p| p.ends_with(".in"))
        .cloned()
        .collect::<Vec<_>>();
    for path in template_paths {
        let contents = fs::read_to_string(&path).expect("Can't read template");
        let processed = regex.replace_all(&contents, |caps: &regex::Captures| {
            env::var(&caps[1]).unwrap_or_default()
        });
        let file_name = Path::new(&path)
            .file_stem()
            .expect("Can't get file stem")
            .to_string_lossy();
        let dest = format!("{}/src-gen/{}", bobje.out_dir_with_target(), file_name);
        write_file_when_different(&dest, &processed).expect("Can't write processed template");
        bobje.source_files.push(dest);
    }
}
