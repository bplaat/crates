/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

// MARK: ToCase
pub(crate) trait ToCase {
    fn to_student_case(&self) -> String;
    fn to_snake_case(&self) -> String;
}

impl ToCase for str {
    fn to_student_case(&self) -> String {
        let mut student_case = String::with_capacity(self.len());
        let mut next_uppercase = true;
        for c in self.chars() {
            if c == '_' {
                next_uppercase = true;
                continue;
            }
            student_case.push(if next_uppercase {
                c.to_ascii_uppercase()
            } else {
                c
            });
            next_uppercase = false;
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
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_to_student_case() {
        assert_eq!("helloWorld".to_student_case(), "HelloWorld");
        assert_eq!("HelloWorld".to_student_case(), "HelloWorld");
        assert_eq!("hello_world".to_student_case(), "HelloWorld");
        assert_eq!("".to_student_case(), "");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!("helloWorld".to_snake_case(), "hello_world");
        assert_eq!("HelloWorld".to_snake_case(), "hello_world");
        assert_eq!("hello_world".to_snake_case(), "hello_world");
        assert_eq!("".to_snake_case(), "");
    }
}
