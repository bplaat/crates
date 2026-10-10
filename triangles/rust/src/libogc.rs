use core::ffi::{c_char, c_int, c_void};

// System and video values
pub const SYS_BASE_CACHED: u32 = 0x8000_0000;
pub const SYS_BASE_UNCACHED: u32 = 0xc000_0000;
pub const VI_NON_INTERLACE: u32 = 1;
pub const CONF_ASPECT_16_9: i32 = 1;
pub const WPAD_CHAN_ALL: i32 = -1;
pub const WPAD_BUTTON_HOME: u32 = 0x8000;

// GX values
pub const GX_MAX_Z24: u32 = 0x00ff_ffff;
pub const GX_ENABLE: u8 = 1;
pub const GX_DISABLE: u8 = 0;
pub const GX_TRIANGLES: u8 = 0x90;
pub const GX_VTXFMT0: u8 = 0;
pub const GX_VA_POS: u8 = 9;
pub const GX_VA_CLR0: u8 = 11;
pub const GX_DIRECT: u8 = 1;
pub const GX_POS_XY: u32 = 0;
pub const GX_S8: u32 = 1;
pub const GX_CLR_RGBA: u32 = 1;
pub const GX_RGBA8: u32 = 5;
pub const GX_COLOR0A0: i32 = 4;
pub const GX_SRC_VTX: u8 = 1;
pub const GX_DF_NONE: u8 = 0;
pub const GX_AF_NONE: u8 = 2;
pub const GX_TEXCOORD_NULL: u8 = 0xff;
pub const GX_TEXMAP_NULL: u32 = 0xff;
pub const GX_PASSCLR: u8 = 4;
pub const GX_CULL_NONE: u8 = 0;
pub const GX_ALWAYS: u8 = 7;
pub const GX_LEQUAL: u8 = 3;
pub const GX_PERSPECTIVE: u8 = 0;
pub const GX_PNMTX0: u32 = 0;
pub const GX_ZC_LINEAR: u8 = 0;
pub const GX_GM_1_0: u8 = 0;
pub const GX_PF_RGB8_Z24: u8 = 0;
pub const GX_PF_RGB565_Z16: u8 = 2;
#[repr(C)]
pub struct GXRModeObj {
    pub vi_tv_mode: u32,
    pub fb_width: u16,
    pub efb_height: u16,
    pub xfb_height: u16,
    pub vi_x_origin: u16,
    pub vi_y_origin: u16,
    pub vi_width: u16,
    pub vi_height: u16,
    pub xfb_mode: u32,
    pub rgb: u8,
    pub field_rendering: u8,
    pub aa: u8,
    pub sample_pattern: [[u8; 2]; 12],
    pub vfilter: [u8; 7],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GXColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub type Mtx = [[f32; 4]; 3];
pub type Mtx44 = [[f32; 4]; 4];

#[repr(C)]
pub struct GXFifoObj {
    _private: [u8; 0],
}

pub type PowerCallback = Option<extern "C" fn()>;
pub type WpadPowerCallback = Option<extern "C" fn(c_int)>;

unsafe extern "C" {
    // C library
    pub fn memalign(alignment: usize, size: usize) -> *mut c_void;
    pub fn free(pointer: *mut c_void);

    // Video and system
    pub fn VIDEO_Init();
    pub fn VIDEO_SetBlack(black: bool);
    pub fn VIDEO_GetPreferredMode(mode: *mut GXRModeObj) -> *mut GXRModeObj;
    pub fn SYS_AllocateFramebuffer(mode: *const GXRModeObj) -> *mut c_void;
    pub fn CONF_GetAspectRatio() -> c_int;
    pub fn VIDEO_Configure(mode: *const GXRModeObj);
    pub fn VIDEO_SetNextFramebuffer(framebuffer: *mut c_void);
    pub fn VIDEO_Flush();
    pub fn VIDEO_WaitVSync();

    // GX video setup and rendering
    pub fn GX_Init(fifo: *mut c_void, size: u32) -> *mut GXFifoObj;
    pub fn GX_SetViewport(x: f32, y: f32, width: f32, height: f32, near: f32, far: f32);
    pub fn GX_SetScissor(x: u32, y: u32, width: u32, height: u32);
    pub fn GX_GetYScaleFactor(efb_height: u16, xfb_height: u16) -> f32;
    pub fn GX_SetDispCopyYScale(scale: f32) -> u32;
    pub fn GX_SetDispCopySrc(x: u16, y: u16, width: u16, height: u16);
    pub fn GX_SetDispCopyDst(width: u16, height: u16);
    pub fn GX_SetCopyFilter(aa: u8, sample_pattern: *const [u8; 2], vf: u8, vfilter: *const u8);
    pub fn GX_SetFieldMode(field_mode: u8, half_aspect_ratio: u8);
    pub fn GX_SetPixelFmt(pixel_format: u8, depth_compression: u8);
    pub fn GX_SetDispCopyGamma(gamma: u8);
    pub fn GX_SetCopyClear(color: GXColor, depth: u32);
    pub fn GX_SetZMode(enable: u8, function: u8, update_enable: u8);
    pub fn GX_CopyDisp(destination: *mut c_void, clear: u8);
    pub fn GX_ClearVtxDesc();
    pub fn GX_SetVtxDesc(attribute: u8, mode: u8);
    pub fn GX_SetVtxAttrFmt(
        format: u8,
        attribute: u32,
        component_type: u32,
        component_size: u32,
        fractional_bits: u32,
    );
    pub fn GX_SetNumChans(count: u8);
    pub fn GX_SetChanCtrl(
        channel: i32,
        enable: u8,
        ambient_source: u8,
        material_source: u8,
        light_mask: u8,
        diffuse_function: u8,
        attenuation_function: u8,
    );
    pub fn GX_SetNumTexGens(count: u32);
    pub fn GX_SetTevOrder(stage: u8, coordinate: u8, map: u32, color: u8);
    pub fn GX_SetTevOp(stage: u8, mode: u8);
    pub fn GX_SetCullMode(mode: u8);
    pub fn DCFlushRange(pointer: *mut c_void, size: u32);

    // Matrix helpers
    pub fn guPerspective(matrix: *mut Mtx44, fovy: f32, aspect: f32, near: f32, far: f32);
    pub fn GX_LoadProjectionMtx(matrix: *const Mtx44, projection_type: u8);
    pub fn ps_guMtxRotRad(matrix: *mut Mtx, axis: c_char, radians: f32);
    pub fn ps_guMtxConcat(a: *const Mtx, b: *const Mtx, destination: *mut Mtx);
    pub fn ps_guMtxTransApply(source: *const Mtx, destination: *mut Mtx, x: f32, y: f32, z: f32);
    pub fn GX_LoadPosMtxImm(matrix: *const Mtx, index: u32);
    pub fn GX_CallDispList(list: *const c_void, size: u32);
    pub fn GX_DrawDone();

    // Wii Remote input and power callbacks
    pub fn WPAD_Init() -> c_int;
    pub fn SYS_SetPowerCallback(callback: PowerCallback) -> PowerCallback;
    pub fn WPAD_SetPowerButtonCallback(callback: WpadPowerCallback);
    pub fn WPAD_ScanPads() -> c_int;
    pub fn WPAD_ButtonsDown(channel: c_int) -> u32;
    pub fn WPAD_Disconnect(channel: c_int) -> c_int;
}
