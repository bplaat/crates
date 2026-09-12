/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Generator for the checked-in image decoder fixtures.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, fs};

fn main() {
    require("magick");

    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixtures = crate_dir.join("tests/fixtures");
    let ico_only = match env::args().nth(1).as_deref() {
        None => false,
        Some("--ico-only") => true,
        Some(argument) => panic!("unknown argument: {argument}"),
    };
    if ico_only {
        write_ico_fixtures(&crate_dir, &fixtures);
        return;
    }

    require("cjpeg");
    let work = TemporaryDirectory::new();
    let source = work.path.join("source.ppm");
    write_source(&source, 19, 13);

    convert(
        &source,
        &["-define", "png:color-type=2", "-depth", "8"],
        &fixtures.join("rgb.png"),
    );
    convert(
        &source,
        &[
            "-alpha",
            "set",
            "-channel",
            "A",
            "-evaluate",
            "set",
            "70%",
            "+channel",
            "-define",
            "png:color-type=6",
            "-depth",
            "8",
        ],
        &fixtures.join("rgba.png"),
    );
    convert(
        &source,
        &[
            "-interlace",
            "PNG",
            "-define",
            "png:color-type=2",
            "-depth",
            "8",
        ],
        &fixtures.join("adam7.png"),
    );
    convert(
        &source,
        &["-colors", "16", "-type", "Palette", "-depth", "8"],
        &fixtures.join("palette.png"),
    );
    convert(
        &source,
        &["-colorspace", "Gray", "-depth", "8"],
        &fixtures.join("gray.png"),
    );
    convert(
        &source,
        &["-sampling-factor", "1x1", "-quality", "90"],
        &fixtures.join("baseline.jpg"),
    );
    convert(
        &source,
        &["-sampling-factor", "2x2", "-quality", "90"],
        &fixtures.join("subsampled.jpg"),
    );
    convert(
        &source,
        &[
            "-sampling-factor",
            "2x2",
            "-interlace",
            "Plane",
            "-quality",
            "90",
        ],
        &fixtures.join("progressive.jpg"),
    );
    convert(
        &source,
        &["-colorspace", "Gray", "-quality", "90"],
        &fixtures.join("gray.jpg"),
    );
    convert(
        &source,
        &["-colorspace", "CMYK", "-quality", "90"],
        &fixtures.join("cmyk.jpg"),
    );
    convert(
        &source,
        &["-type", "TrueColor", "-define", "bmp:format=bmp3"],
        &fixtures.join("rgb.bmp"),
    );
    convert(
        &source,
        &[
            "-colors",
            "16",
            "-type",
            "Palette",
            "-define",
            "bmp:format=bmp3",
        ],
        &fixtures.join("palette.bmp"),
    );
    convert(&source, &["-colors", "32"], &fixtures.join("static.gif"));
    convert(
        &source,
        &["-colors", "32", "-interlace", "GIF"],
        &fixtures.join("interlaced.gif"),
    );
    convert(&fixtures.join("rgb.png"), &[], &fixtures.join("rgb.qoi"));
    convert(&fixtures.join("rgba.png"), &[], &fixtures.join("rgba.qoi"));
    write_qoi_operations(&fixtures.join("operations.qoi"));

    for name in [
        "rgb.png",
        "rgba.png",
        "adam7.png",
        "palette.png",
        "gray.png",
        "baseline.jpg",
        "subsampled.jpg",
        "progressive.jpg",
        "gray.jpg",
        "cmyk.jpg",
        "rgb.bmp",
        "palette.bmp",
        "static.gif",
        "interlaced.gif",
        "rgb.qoi",
        "rgba.qoi",
        "operations.qoi",
    ] {
        reference(&fixtures, name);
    }

    run(Command::new("magick")
        .args(["-delay", "7"])
        .arg(&source)
        .args(["-delay", "13", "("])
        .arg(&source)
        .args(["-negate", ")", "-loop", "3"])
        .arg(fixtures.join("animated.gif")));
    run(Command::new("magick")
        .arg(fixtures.join("animated.gif"))
        .args(["-coalesce", "-alpha", "on", "-depth", "8"])
        .arg(raw_path(&fixtures.join("animated.gif.rgba"))));
    let scans = work.path.join("scans.txt");
    fs::write(&scans, "0; 1; 2;\n").expect("write JPEG scan script");
    cjpeg(
        &source,
        &["-quality", "90", "-restart", "1B"],
        &fixtures.join("restart.jpg"),
    );
    cjpeg(
        &source,
        &["-quality", "90", "-progressive", "-restart", "1B"],
        &fixtures.join("progressive-restart.jpg"),
    );
    run(Command::new("cjpeg")
        .args(["-quality", "90", "-scans"])
        .arg(&scans)
        .arg("-outfile")
        .arg(fixtures.join("separate.jpg"))
        .arg(&source));
    cjpeg(
        &source,
        &["-quality", "90", "-rgb"],
        &fixtures.join("direct-rgb.jpg"),
    );
    cjpeg(
        &source,
        &["-quality", "90", "-sample", "2x1,1x1,1x1"],
        &fixtures.join("horizontal.jpg"),
    );
    cjpeg(
        &source,
        &["-quality", "90", "-sample", "1x2,1x1,1x1"],
        &fixtures.join("vertical.jpg"),
    );
    for name in [
        "restart.jpg",
        "progressive-restart.jpg",
        "separate.jpg",
        "direct-rgb.jpg",
        "horizontal.jpg",
        "vertical.jpg",
    ] {
        reference(&fixtures, name);
    }

    let large = work.path.join("large.ppm");
    write_noise(&large, 97, 89);
    convert(&large, &[], &fixtures.join("large.gif"));
    reference(&fixtures, "large.gif");
    write_png_fixtures(&fixtures);
    write_ico_fixtures(&crate_dir, &fixtures);
}

fn require(program: &str) {
    let available = Command::new(program)
        .arg("-version")
        .output()
        .is_ok_and(|output| output.status.success());
    assert!(available, "fixture generation requires {program}");
}

fn run(command: &mut Command) {
    let program = command.get_program().to_string_lossy().into_owned();
    let status = command
        .status()
        .unwrap_or_else(|error| panic!("start {program}: {error}"));
    assert!(status.success(), "{program} failed with {status}");
}

fn convert(input: &Path, arguments: &[&str], output: &Path) {
    run(Command::new("magick")
        .arg(input)
        .args(arguments)
        .arg(output));
}

fn reference(fixtures: &Path, name: &str) {
    let input = fixtures.join(name);
    let mut output = input.as_os_str().to_owned();
    output.push(".rgba");
    run(Command::new("magick")
        .arg(input)
        .args(["-colorspace", "sRGB", "-alpha", "on", "-depth", "8"])
        .arg(raw_path(Path::new(&output))));
}

fn reference_ico(fixtures: &Path, name: &str, frame: usize) {
    let input = format!("{}[{frame}]", fixtures.join(name).display());
    run(Command::new("magick")
        .arg(input)
        .args(["-colorspace", "sRGB", "-alpha", "on", "-depth", "8"])
        .arg(raw_path(&fixtures.join(format!("{name}.rgba")))));
}

fn write_ico_fixtures(crate_dir: &Path, fixtures: &Path) {
    let source = fixtures.join("rgb.png");
    run(Command::new("magick")
        .arg("(")
        .arg(&source)
        .args(["-resize", "16x16!"])
        .arg(")")
        .arg("(")
        .arg(&source)
        .args(["-resize", "32x32!"])
        .arg(")")
        .arg(fixtures.join("dib.ico")));
    reference_ico(fixtures, "dib.ico", 1);

    convert(
        &source,
        &["-colors", "16", "-type", "Palette"],
        &fixtures.join("palette.ico"),
    );
    reference_ico(fixtures, "palette.ico", 0);

    let png = fs::read(&source).expect("read PNG fixture");
    let mut icon = vec![0, 0, 1, 0, 1, 0, 19, 13, 0, 0];
    icon.extend_from_slice(&1u16.to_le_bytes());
    icon.extend_from_slice(&32u16.to_le_bytes());
    icon.extend_from_slice(&(png.len() as u32).to_le_bytes());
    icon.extend_from_slice(&22u32.to_le_bytes());
    icon.extend_from_slice(&png);
    fs::write(fixtures.join("png.ico"), icon).expect("write PNG icon fixture");
    fs::copy(fixtures.join("rgb.png.rgba"), fixtures.join("png.ico.rgba"))
        .expect("write PNG icon reference");

    fs::copy(
        fixtures.join("dib.ico"),
        crate_dir.join("../macview-appkit/tests/fixtures/dib.ico"),
    )
    .expect("write MacView icon fixture");
}

fn raw_path(path: &Path) -> OsString {
    let mut output = OsString::from("rgba:");
    output.push(path);
    output
}

fn cjpeg(input: &Path, arguments: &[&str], output: &Path) {
    run(Command::new("cjpeg")
        .args(arguments)
        .arg("-outfile")
        .arg(output)
        .arg(input));
}

fn write_source(path: &Path, width: u32, height: u32) {
    let mut bytes = format!("P6\n{width} {height}\n255\n").into_bytes();
    for y in 0..height {
        for x in 0..width {
            bytes.extend_from_slice(&[
                (x * 13 + y * 3) as u8,
                (y * 19 + x * 2) as u8,
                (x * 7 + y * 11) as u8,
            ]);
        }
    }
    fs::write(path, bytes).expect("write source image");
}

fn write_noise(path: &Path, width: u32, height: u32) {
    let mut bytes = format!("P6\n{width} {height}\n255\n").into_bytes();
    let mut state = 12_345u32;
    for _ in 0..width * height * 3 {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        bytes.push((state >> 24) as u8);
    }
    fs::write(path, bytes).expect("write noise image");
}

fn write_qoi_operations(path: &Path) {
    let mut bytes = b"qoif".to_vec();
    bytes.extend_from_slice(&19u32.to_be_bytes());
    bytes.extend_from_slice(&13u32.to_be_bytes());
    bytes.extend_from_slice(&[4, 0]);
    bytes.extend_from_slice(&[
        0xfe, 10, 20, 30, 0xff, 1, 2, 3, 4, 0x7f, 0xa2, 0x79, 0xc1, 0x00, 0xfd, 0xfd, 0xfd, 0xf5,
    ]);
    bytes.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 1]);
    fs::write(path, bytes).expect("write QOI operations fixture");
}

fn write_png_fixtures(fixtures: &Path) {
    for depth in [8, 16] {
        let mut header = b"\x89PNG\r\n\x1a\n".to_vec();
        let mut ihdr = Vec::new();
        ihdr.extend_from_slice(&1u32.to_be_bytes());
        ihdr.extend_from_slice(&1u32.to_be_bytes());
        ihdr.extend_from_slice(&[depth, 6, 0, 0, 0]);
        png_chunk(&mut header, b"IHDR", &ihdr);
        let (red, green): (&[u8], &[u8]) = if depth == 8 {
            (&[255, 0, 0, 255], &[0, 255, 0, 255])
        } else {
            (
                &[255, 255, 0, 0, 0, 0, 255, 255],
                &[0, 0, 255, 255, 0, 0, 255, 255],
            )
        };
        let mut animation = header;
        let mut animation_control = Vec::new();
        animation_control.extend_from_slice(&2u32.to_be_bytes());
        animation_control.extend_from_slice(&3u32.to_be_bytes());
        png_chunk(&mut animation, b"acTL", &animation_control);
        png_chunk(&mut animation, b"fcTL", &frame_control(0, 7));
        png_chunk(&mut animation, b"IDAT", &zlib(&[&[0], red].concat()));
        png_chunk(&mut animation, b"fcTL", &frame_control(1, 13));
        let mut frame_data = 2u32.to_be_bytes().to_vec();
        frame_data.extend_from_slice(&zlib(&[&[0], green].concat()));
        png_chunk(&mut animation, b"fdAT", &frame_data);
        png_chunk(&mut animation, b"IEND", &[]);
        let name = if depth == 8 {
            "animated.png"
        } else {
            "16bit-apng.png"
        };
        fs::write(fixtures.join(name), animation).expect("write APNG");
    }
}

fn frame_control(sequence: u32, delay: u16) -> Vec<u8> {
    let mut bytes = Vec::new();
    for value in [sequence, 1, 1, 0, 0] {
        bytes.extend_from_slice(&value.to_be_bytes());
    }
    bytes.extend_from_slice(&delay.to_be_bytes());
    bytes.extend_from_slice(&100u16.to_be_bytes());
    bytes.extend_from_slice(&[0, 0]);
    bytes
}

fn png_chunk(output: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    output.extend_from_slice(&(data.len() as u32).to_be_bytes());
    output.extend_from_slice(kind);
    output.extend_from_slice(data);
    let mut checked = kind.to_vec();
    checked.extend_from_slice(data);
    output.extend_from_slice(&crc32(&checked).to_be_bytes());
}

fn zlib(data: &[u8]) -> Vec<u8> {
    assert!(
        data.len() <= u16::MAX as usize,
        "fixture scanline is too large"
    );
    let mut output = vec![0x78, 0x01];
    output.push(1);
    let len = data.len() as u16;
    output.extend_from_slice(&len.to_le_bytes());
    output.extend_from_slice(&(!len).to_le_bytes());
    output.extend_from_slice(data);
    output.extend_from_slice(&adler32(data).to_be_bytes());
    output
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb8_8320 & 0u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}

fn adler32(data: &[u8]) -> u32 {
    let mut a = 1u32;
    let mut b = 0u32;
    for &byte in data {
        a = (a + u32::from(byte)) % 65_521;
        b = (b + a) % 65_521;
    }
    b << 16 | a
}

struct TemporaryDirectory {
    path: PathBuf,
}

impl TemporaryDirectory {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let path = env::temp_dir().join(format!("image-fixtures-{}-{nonce}", std::process::id()));
        fs::create_dir(&path).expect("create temporary directory");
        Self { path }
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
