/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

pub(crate) trait ToCapitalize {
    fn to_capitalize(&self) -> String;
}

impl ToCapitalize for String {
    fn to_capitalize(&self) -> String {
        let mut c = self.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }
}
