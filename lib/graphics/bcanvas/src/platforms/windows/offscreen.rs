/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::c_void;
use std::ptr::{NonNull, null_mut};

use super::com::ComPtr;
use super::context::PlatformCanvasContext;
use super::headers::*;
use crate::copy_bgra_premultiplied_to_rgba;

#[repr(C)]
#[derive(Default)]
struct BitmapInfoHeader {
    size: u32,
    width: i32,
    height: i32,
    planes: u16,
    bit_count: u16,
    compression: u32,
    size_image: u32,
    x_pixels_per_meter: i32,
    y_pixels_per_meter: i32,
    colors_used: u32,
    colors_important: u32,
}

#[repr(C)]
#[derive(Default)]
struct BitmapInfo {
    header: BitmapInfoHeader,
    colors: [u32; 3],
}

#[link(name = "gdi32")]
unsafe extern "system" {
    fn CreateCompatibleDC(dc: HDC) -> HDC;
    fn CreateDIBSection(
        dc: HDC,
        info: *const BitmapInfo,
        usage: u32,
        bits: *mut *mut c_void,
        section: HANDLE,
        offset: u32,
    ) -> HANDLE;
    fn SelectObject(dc: HDC, object: HANDLE) -> HANDLE;
    fn DeleteDC(dc: HDC) -> BOOL;
}

pub(crate) struct PlatformOffscreenCanvas {
    target: Option<ComPtr<ID2D1HwndRenderTarget>>,
    factory: ComPtr<ID2D1Factory>,
    write: ComPtr<IDWriteFactory>,
    dc: HDC,
    bitmap: HANDLE,
    previous: HANDLE,
    storage: NonNull<u8>,
    length: usize,
    output: Vec<u8>,
}

impl PlatformOffscreenCanvas {
    pub(crate) fn new(width: u32, height: u32, length: usize) -> Self {
        unsafe {
            let dc = CreateCompatibleDC(null_mut());
            assert!(!dc.is_null(), "Could not create bitmap device context");
            let info = BitmapInfo {
                header: BitmapInfoHeader {
                    size: size_of::<BitmapInfoHeader>() as u32,
                    width: width as i32,
                    height: -(height as i32),
                    planes: 1,
                    bit_count: 32,
                    size_image: length as u32,
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut bits = null_mut();
            let bitmap = CreateDIBSection(dc, &info, 0, &mut bits, null_mut(), 0);
            assert!(!bitmap.is_null(), "Could not create bitmap storage");
            let storage = NonNull::new(bits.cast()).expect("Bitmap storage was null");
            let previous = SelectObject(dc, bitmap);
            assert!(!previous.is_null(), "Could not select bitmap storage");
            let factory = ID2D1Factory::new().expect("Direct2D factory");
            let write = IDWriteFactory::new().expect("DirectWrite factory");
            let target = factory
                .CreateDCRenderTarget(&D2D1_RENDER_TARGET_PROPERTIES {
                    pixelFormat: D2D1_PIXEL_FORMAT {
                        format: DXGI_FORMAT_B8G8R8A8_UNORM,
                        alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                    },
                    dpiX: 96.0,
                    dpiY: 96.0,
                    ..Default::default()
                })
                .expect("Direct2D bitmap target");
            target
                .BindDC(
                    dc,
                    &RECT {
                        left: 0,
                        top: 0,
                        right: width as i32,
                        bottom: height as i32,
                    },
                )
                .expect("Bind Direct2D bitmap target");
            Self {
                target: Some(target),
                factory,
                write,
                dc,
                bitmap,
                previous,
                storage,
                length,
                output: vec![0; length],
            }
        }
    }

    pub(crate) fn begin_draw(&mut self) -> PlatformCanvasContext {
        let target = self.target.as_ref().expect("live bitmap target");
        unsafe { target.BeginDraw() };
        PlatformCanvasContext::new(target.clone(), self.factory.clone(), self.write.clone())
    }

    pub(crate) fn end_draw(&mut self) {
        unsafe {
            self.target
                .as_ref()
                .expect("live bitmap target")
                .EndDraw(null_mut(), null_mut())
                .expect("Draw offscreen canvas");
        }
    }

    pub(crate) fn pixels(&mut self) -> &[u8] {
        let storage = unsafe { std::slice::from_raw_parts(self.storage.as_ptr(), self.length) };
        copy_bgra_premultiplied_to_rgba(storage, &mut self.output);
        &self.output
    }
}

impl Drop for PlatformOffscreenCanvas {
    fn drop(&mut self) {
        self.target.take();
        unsafe {
            SelectObject(self.dc, self.previous);
            DeleteObject(self.bitmap);
            DeleteDC(self.dc);
        }
    }
}
