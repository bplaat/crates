/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

#[path = "../../../../tools/wii-toolchain/build.rs"]
mod wii;

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let assets = manifest_dir.join("src/assets");
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let script = assets.join("materials.scf");
    let tpl = out_dir.join("materials.tpl");
    let image_data = out_dir.join("materials.bin");
    let dependencies = out_dir.join("materials.d");

    println!("cargo:rerun-if-changed={}", script.display());
    let script_text = fs::read_to_string(&script).expect("failed to read materials.scf");
    let image_names: Vec<_> = script_text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.split_once("filepath=\"")
                .and_then(|(_, rest)| rest.split_once('"'))
                .map(|(name, _)| name)
                .expect("invalid materials.scf entry")
        })
        .collect();
    assert!(!image_names.is_empty(), "materials.scf has no images");
    for &image in &image_names {
        println!("cargo:rerun-if-changed={}", assets.join(image).display());
    }
    let toolchain_build = wii::build(manifest_dir.join("../../../../tools/wii-toolchain"));
    let status = Command::new(toolchain_build.join("gxtexconv"))
        .arg("-s")
        .arg(&script)
        .arg("-d")
        .arg(&dependencies)
        .arg("-o")
        .arg(&tpl)
        .status()
        .expect("failed to run gxtexconv");
    assert!(status.success(), "gxtexconv failed to build materials.tpl");

    let tpl_data = fs::read(&tpl).expect("failed to read materials.tpl");
    let read_u32 = |offset: usize| {
        u32::from_be_bytes(
            tpl_data
                .get(offset..offset + 4)
                .expect("truncated materials.tpl")
                .try_into()
                .unwrap(),
        ) as usize
    };
    assert_eq!(read_u32(0), 0x0020_af30, "invalid TPL signature");
    assert_eq!(read_u32(4), image_names.len(), "unexpected material count");
    assert_eq!(read_u32(8), 12, "unexpected TPL table offset");

    const IMAGE_SIZE: usize = 128 * 128 / 2;
    let mut images = Vec::with_capacity(image_names.len() * IMAGE_SIZE);
    for index in 0..image_names.len() {
        let header = read_u32(12 + index * 8);
        assert_eq!(read_u32(16 + index * 8), 0, "unexpected palette");
        let dimensions = tpl_data
            .get(header..header + 4)
            .expect("truncated texture header");
        assert_eq!(u16::from_be_bytes([dimensions[0], dimensions[1]]), 128);
        assert_eq!(u16::from_be_bytes([dimensions[2], dimensions[3]]), 128);
        assert_eq!(read_u32(header + 4), 14, "expected GX CMPR texture");
        assert_eq!(read_u32(header + 12), 1, "expected S repeat wrap");
        assert_eq!(read_u32(header + 16), 1, "expected T repeat wrap");
        assert_eq!(read_u32(header + 20), 1, "expected linear min filter");
        assert_eq!(read_u32(header + 24), 1, "expected linear mag filter");
        assert_eq!(read_u32(header + 28), 0, "unexpected LOD bias");
        assert_eq!(tpl_data.get(header + 32..header + 34), Some(&[0, 0][..]));
        let image = read_u32(header + 8);
        images.extend_from_slice(
            tpl_data
                .get(image..image + IMAGE_SIZE)
                .expect("truncated texture data"),
        );
    }
    fs::write(image_data, images).expect("failed to write materials.bin");
}
