/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [copy_dir](https://crates.io/crates/copy_dir) crate

use std::path::Path;
use std::{fs, io};

/// Recursively copies all files and directories from `from` to `to`.
pub fn copy_dir(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
    let src = from.as_ref();
    let dst = to.as_ref();
    let source_type = fs::symlink_metadata(src)?.file_type();
    if !source_type.is_dir() || source_type.is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "source must be a directory and not a symbolic link",
        ));
    }
    let canonical_src = fs::canonicalize(src)?;
    let resolved_dst = resolve_destination(dst)?;
    if resolved_dst == canonical_src || resolved_dst.starts_with(&canonical_src) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "destination must not be the source or inside it",
        ));
    }

    if dst.exists() {
        fs::remove_dir_all(dst)?;
    }
    fs::create_dir_all(dst)?;

    fn copy_dir(src: &Path, dst: &Path) -> io::Result<()> {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let dst_path = dst.join(entry.file_name());
            let file_type = entry.file_type()?;
            if file_type.is_symlink() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "symbolic links are not supported: {}",
                        entry.path().display()
                    ),
                ));
            }
            if file_type.is_dir() {
                copy_dir(&entry.path(), &dst_path)?;
            } else if file_type.is_file() {
                fs::copy(entry.path(), dst_path)?;
            }
        }
        Ok(())
    }
    copy_dir(src, dst)?;
    Ok(())
}

fn resolve_destination(path: &Path) -> io::Result<std::path::PathBuf> {
    let absolute = std::path::absolute(path)?;
    let existing = absolute
        .ancestors()
        .find(|ancestor| ancestor.exists())
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "destination has no ancestor"))?;
    let suffix = absolute
        .strip_prefix(existing)
        .expect("existing ancestor must prefix destination");
    Ok(fs::canonicalize(existing)?.join(suffix))
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    fn temp(suffix: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("copy_dir_{suffix}"))
    }

    #[test]
    fn test_copy_flat_dir() {
        let src = temp("flat_src");
        let dst = temp("flat_dst");
        let _ = fs::remove_dir_all(&src);
        let _ = fs::remove_dir_all(&dst);

        fs::create_dir(&src).unwrap();
        fs::write(src.join("a.txt"), "hello").unwrap();
        fs::write(src.join("b.txt"), "world").unwrap();

        copy_dir(&src, &dst).unwrap();

        assert_eq!(fs::read_to_string(dst.join("a.txt")).unwrap(), "hello");
        assert_eq!(fs::read_to_string(dst.join("b.txt")).unwrap(), "world");
        assert!(!dst.join("c.txt").exists());

        let _ = fs::remove_dir_all(&src);
        let _ = fs::remove_dir_all(&dst);
    }

    #[test]
    fn test_copy_nested_dir() {
        let src = temp("nested_src");
        let dst = temp("nested_dst");
        let _ = fs::remove_dir_all(&src);
        let _ = fs::remove_dir_all(&dst);

        fs::create_dir_all(src.join("sub")).unwrap();
        fs::write(src.join("root.txt"), "root").unwrap();
        fs::write(src.join("sub").join("child.txt"), "child").unwrap();

        copy_dir(&src, &dst).unwrap();

        assert_eq!(fs::read_to_string(dst.join("root.txt")).unwrap(), "root");
        assert_eq!(
            fs::read_to_string(dst.join("sub").join("child.txt")).unwrap(),
            "child"
        );

        let _ = fs::remove_dir_all(&src);
        let _ = fs::remove_dir_all(&dst);
    }

    #[test]
    fn test_copy_overwrites_existing_destination() {
        let src = temp("overwrite_src");
        let dst = temp("overwrite_dst");
        let _ = fs::remove_dir_all(&src);
        let _ = fs::remove_dir_all(&dst);

        // Pre-populate destination with a file that should be gone after copy
        fs::create_dir(&dst).unwrap();
        fs::write(dst.join("old.txt"), "stale").unwrap();

        fs::create_dir(&src).unwrap();
        fs::write(src.join("new.txt"), "fresh").unwrap();

        copy_dir(&src, &dst).unwrap();

        assert!(!dst.join("old.txt").exists()); // wiped by copy_dir
        assert_eq!(fs::read_to_string(dst.join("new.txt")).unwrap(), "fresh");

        let _ = fs::remove_dir_all(&src);
        let _ = fs::remove_dir_all(&dst);
    }

    #[test]
    fn test_rejects_destination_inside_source() {
        let src = temp("nested_destination_src");
        let _ = fs::remove_dir_all(&src);
        fs::create_dir(&src).unwrap();
        fs::write(src.join("keep.txt"), "keep").unwrap();

        assert_eq!(
            copy_dir(&src, src.join("copy")).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
        assert_eq!(fs::read_to_string(src.join("keep.txt")).unwrap(), "keep");

        let _ = fs::remove_dir_all(&src);
    }

    #[test]
    fn test_rejects_source_as_destination() {
        let src = temp("same_destination_src");
        let _ = fs::remove_dir_all(&src);
        fs::create_dir(&src).unwrap();
        fs::write(src.join("keep.txt"), "keep").unwrap();

        assert_eq!(
            copy_dir(&src, &src).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
        assert_eq!(fs::read_to_string(src.join("keep.txt")).unwrap(), "keep");

        let _ = fs::remove_dir_all(&src);
    }

    #[cfg(unix)]
    #[test]
    fn test_rejects_source_symlinks() {
        use std::os::unix::fs::symlink;

        let src = temp("symlink_src");
        let dst = temp("symlink_dst");
        let outside = temp("symlink_outside");
        let _ = fs::remove_dir_all(&src);
        let _ = fs::remove_dir_all(&dst);
        let _ = fs::remove_file(&outside);
        fs::create_dir(&src).unwrap();
        fs::write(&outside, "secret").unwrap();
        symlink(&outside, src.join("secret.txt")).unwrap();

        assert_eq!(
            copy_dir(&src, &dst).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
        assert!(!dst.join("secret.txt").exists());

        let _ = fs::remove_dir_all(&src);
        let _ = fs::remove_dir_all(&dst);
        let _ = fs::remove_file(&outside);
    }
}
