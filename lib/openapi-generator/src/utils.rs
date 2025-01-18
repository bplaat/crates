/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

pub(crate) trait ToCase {
    fn to_student_case(&self) -> String;
    fn to_snake_case(&self) -> String;
    fn to_scream_case(&self) -> String;
}

impl ToCase for str {
    fn to_student_case(&self) -> String {
        let mut student_case = String::with_capacity(self.len());
        for (i, c) in self.chars().enumerate() {
            student_case.push(if i == 0 { c.to_ascii_uppercase() } else { c });
        }
        student_case
    }

    fn to_snake_case(&self) -> String {
        let mut snake_case = String::with_capacity(self.len());
        for (i, c) in self.chars().enumerate() {
            if c.is_uppercase() && i != 0 {
                snake_case.push('_');
            }
            snake_case.push(c.to_ascii_lowercase());
        }
        snake_case
    }

    fn to_scream_case(&self) -> String {
        let mut scream_case = String::with_capacity(self.len());
        for (i, c) in self.chars().enumerate() {
            if c.is_uppercase() && i != 0 {
                scream_case.push('_');
            }
            scream_case.push(c.to_ascii_uppercase());
        }
        scream_case
    }
}
