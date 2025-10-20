/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

// MARK: Name
pub(crate) fn name_validator(name: &str) -> validate::Result {
    if name.to_lowercase() == "bastiaan" {
        Err(validate::Error::new("name can't be Bastiaan"))
    } else {
        Ok(())
    }
}
