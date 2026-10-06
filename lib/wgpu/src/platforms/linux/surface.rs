/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Vulkan swapchain presentation for bwindow GTK content widgets.

use std::cell::{Cell, RefCell};
use std::ffi::{c_ulong, c_void};
use std::ptr;
use std::rc::Rc;

use super::*;
use crate::platform::headers::{
    gdk_wayland_display_get_wl_display, gdk_wayland_window_get_wl_surface, gdk_window_get_display,
    gdk_x11_display_get_xdisplay, gdk_x11_window_get_xid, gtk_widget_add_tick_callback,
    gtk_widget_get_scale_factor, gtk_widget_get_window, gtk_widget_remove_tick_callback,
};

pub(super) enum PlatformWindow {
    Xlib {
        display: *mut c_void,
        window: c_ulong,
    },
    Wayland {
        display: *mut c_void,
        surface: *mut c_void,
    },
}

pub(super) fn platform_window(handle: bwindow::NativeWindowHandle) -> Result<PlatformWindow> {
    let bwindow::NativeWindowHandle::Gtk(widget) = handle else {
        return Err(Error("Vulkan requires a GTK window".into()));
    };
    unsafe {
        let window = gtk_widget_get_window(widget);
        if window.is_null() {
            return Err(Error("GTK window is not realized".into()));
        }
        let display = gdk_window_get_display(window);
        if display.is_null() {
            return Err(Error("GTK window has no display".into()));
        }
        if vk::g_type_check_instance_is_a(display, vk::gdk_wayland_display_get_type()) != 0 {
            let display = gdk_wayland_display_get_wl_display(display);
            let surface = gdk_wayland_window_get_wl_surface(window);
            if display.is_null() || surface.is_null() {
                return Err(Error("GTK returned invalid Wayland handles".into()));
            }
            Ok(PlatformWindow::Wayland { display, surface })
        } else {
            let display = gdk_x11_display_get_xdisplay(display);
            let window = gdk_x11_window_get_xid(window);
            if display.is_null() || window == 0 {
                return Err(Error("GTK returned invalid X11 handles".into()));
            }
            Ok(PlatformWindow::Xlib { display, window })
        }
    }
}

const fn from_format(value: u32) -> Option<TextureFormat> {
    match value {
        37 => Some(TextureFormat::Rgba8Unorm),
        43 => Some(TextureFormat::Rgba8UnormSrgb),
        44 => Some(TextureFormat::Bgra8Unorm),
        50 => Some(TextureFormat::Bgra8UnormSrgb),
        _ => None,
    }
}

pub(crate) type Factory = Rc<Instance>;

struct SurfaceOwner {
    instance: Rc<Instance>,
    handle: u64,
    widget: Rc<widget::Widget>,
}

impl Drop for SurfaceOwner {
    fn drop(&mut self) {
        unsafe {
            call!(
                self.instance,
                DestroySurfaceKHR,
                self.instance.handle,
                self.handle,
                ptr::null()
            );
        }
    }
}

pub(crate) struct Surface {
    instance: Rc<Instance>,
    handle: u64,
    owner: Rc<SurfaceOwner>,
    swap: RefCell<Option<Rc<Swapchain>>>,
    pub(super) widget: *mut c_void,
    pub(super) redraw: Rc<RedrawTimer>,
    pub(super) startup_clock: *mut c_void,
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            if !self.startup_clock.is_null() {
                bwindow::ffi::g_signal_handlers_disconnect_matched(
                    self.startup_clock.cast(),
                    16, // G_SIGNAL_MATCH_DATA
                    0,
                    0,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    Rc::as_ptr(&self.redraw) as *mut c_void,
                );
                bwindow::ffi::g_object_unref(self.startup_clock.cast());
            }
            let window = gtk_widget_get_window(self.widget);
            if !window.is_null() {
                vk::gdk_window_set_invalidate_handler(window, None);
                vk::g_object_set_data(window, c"bplaat-wgpu-redraw".as_ptr(), ptr::null_mut());
            }
            bwindow::ffi::g_signal_handlers_disconnect_matched(
                self.widget.cast(),
                16, // G_SIGNAL_MATCH_DATA
                0,
                0,
                ptr::null_mut(),
                ptr::null_mut(),
                Rc::as_ptr(&self.redraw) as *mut c_void,
            );
        }
        if let Some(id) = self.redraw.callback.take() {
            unsafe { gtk_widget_remove_tick_callback(self.widget, id) };
        }
    }
}

pub(super) struct RedrawTimer {
    widget: *mut c_void,
    sender: bwindow::WindowEventSender,
    pub(super) callback: Cell<Option<u32>>,
    ready: Cell<bool>,
}

extern "C" fn surface_after_paint(_: *mut c_void, data: *const RedrawTimer) {
    let timer = unsafe {
        Rc::increment_strong_count(data);
        Rc::from_raw(data)
    };
    if !timer.ready.replace(true) {
        timer.request_animation_frame();
    }
}

impl Drop for RedrawTimer {
    fn drop(&mut self) {
        if let Some(id) = self.callback.take() {
            unsafe { gtk_widget_remove_tick_callback(self.widget, id) };
        }
    }
}

impl RedrawTimer {
    fn request_animation_frame(self: &Rc<Self>) {
        if self.callback.get().is_some() {
            return;
        }
        let data = Rc::into_raw(self.clone()) as *mut c_void;
        let id =
            unsafe { gtk_widget_add_tick_callback(self.widget, redraw_tick, data, release_redraw) };
        self.callback.set(Some(id));
    }
}

extern "C" fn surface_invalidate(window: *mut c_void, region: *mut c_void) {
    unsafe {
        // This native window belongs to Vulkan. Preserve the repaint request,
        // but remove its Cairo damage so GDK cannot attach a competing buffer.
        let pointer =
            vk::g_object_get_data(window, c"bplaat-wgpu-redraw".as_ptr()).cast::<RedrawTimer>();
        if !pointer.is_null() {
            Rc::increment_strong_count(pointer);
            let timer = Rc::from_raw(pointer);
            timer.request_animation_frame();
        }
        vk::cairo_region_subtract(region, region);
    }
}

extern "C" fn surface_draw(_: *mut c_void, _: *mut c_void, data: *const RedrawTimer) -> i32 {
    // GTK flushes its backing buffer after this signal, which can overwrite a
    // Vulkan presentation. Repaint on the next tick, after GTK finishes drawing.
    let timer = unsafe {
        Rc::increment_strong_count(data);
        Rc::from_raw(data)
    };
    timer.request_animation_frame();
    1
}

extern "C" fn redraw_tick(_: *mut c_void, _: *mut c_void, data: *mut c_void) -> i32 {
    // The application can close the window (removing this callback) while
    // handling the redraw. Retain the callback data across that delivery.
    let timer = unsafe {
        let pointer = data.cast::<RedrawTimer>();
        Rc::increment_strong_count(pointer);
        Rc::from_raw(pointer)
    };
    timer.callback.set(None);
    timer.sender.send(bwindow::WindowEvent::RedrawRequested);
    0
}

extern "C" fn release_redraw(data: *mut c_void) {
    drop(unsafe { Rc::from_raw(data.cast::<RedrawTimer>()) });
}

impl Surface {
    pub(crate) fn new(
        window: bwindow::NativeWindowHandle,
        sender: bwindow::WindowEventSender,
        factory: &mut Option<Factory>,
    ) -> Result<Self> {
        unsafe {
            let bwindow::NativeWindowHandle::Gtk(widget) = window else {
                return Err(Error("Vulkan requires a GTK window".into()));
            };
            let content = widget::Widget::new(widget, sender.clone());
            let parent = widget;
            let widget = content.handle;
            let window = platform_window(bwindow::NativeWindowHandle::Gtk(widget))?;
            let wayland = matches!(window, PlatformWindow::Wayland { .. });
            let platform = match window {
                PlatformWindow::Xlib { .. } => c"VK_KHR_xlib_surface",
                PlatformWindow::Wayland { .. } => c"VK_KHR_wayland_surface",
            };
            let instance = if let Some(instance) = factory {
                instance.clone()
            } else {
                let instance = Instance::new(Some(platform))?;
                *factory = Some(instance.clone());
                instance
            };
            let handle = instance.handle;
            let mut surface = 0;
            match window {
                PlatformWindow::Xlib { display, window } => {
                    let info = vk::XlibSurfaceCreateInfoKHR {
                        s_type: 1000004000,
                        dpy: display,
                        window,
                        ..Default::default()
                    };
                    check(call!(
                        instance,
                        CreateXlibSurfaceKHR,
                        handle,
                        &info,
                        ptr::null(),
                        &mut surface
                    ))?;
                }
                PlatformWindow::Wayland {
                    display,
                    surface: window,
                } => {
                    let info = vk::WaylandSurfaceCreateInfoKHR {
                        s_type: 1000006000,
                        display,
                        surface: window,
                        ..Default::default()
                    };
                    check(call!(
                        instance,
                        CreateWaylandSurfaceKHR,
                        handle,
                        &info,
                        ptr::null(),
                        &mut surface
                    ))?;
                }
            }
            let redraw = Rc::new(RedrawTimer {
                widget,
                sender,
                callback: Cell::new(None),
                ready: Cell::new(!wayland),
            });
            // GDK keeps a new Wayland subsurface synchronized until the parent
            // commits its first frame. Presenting repeatedly before that commit
            // can merge Vulkan FIFO timestamps and cause a fatal protocol error.
            let startup_clock = if wayland {
                let clock = vk::gtk_widget_get_frame_clock(parent);
                bwindow::ffi::g_object_ref(clock.cast());
                bwindow::ffi::g_signal_connect_data(
                    clock.cast(),
                    c"after-paint".as_ptr(),
                    surface_after_paint as *const c_void,
                    Rc::as_ptr(&redraw).cast(),
                    ptr::null(),
                    1, // G_CONNECT_AFTER: GDK must commit and desynchronize first.
                );
                clock
            } else {
                ptr::null_mut()
            };
            let native_window = gtk_widget_get_window(widget);
            vk::g_object_set_data(
                native_window,
                c"bplaat-wgpu-redraw".as_ptr(),
                Rc::as_ptr(&redraw) as *mut c_void,
            );
            vk::gdk_window_set_invalidate_handler(native_window, Some(surface_invalidate));
            // Discard Cairo damage queued while GTK realized and mapped the widget.
            let pending = vk::gdk_window_get_update_area(native_window);
            if !pending.is_null() {
                vk::cairo_region_destroy(pending);
            }
            bwindow::ffi::g_signal_connect_data(
                widget.cast(),
                c"draw".as_ptr(),
                surface_draw as *const c_void,
                Rc::as_ptr(&redraw).cast(),
                ptr::null(),
                bwindow::ffi::G_CONNECT_DEFAULT,
            );
            Ok(Self {
                owner: Rc::new(SurfaceOwner {
                    instance: instance.clone(),
                    handle: surface,
                    widget: content,
                }),
                instance,
                handle: surface,
                swap: RefCell::new(None),
                widget,
                redraw,
                startup_clock,
            })
        }
    }

    pub(crate) fn request_redraw(&self) {
        self.redraw
            .sender
            .send(bwindow::WindowEvent::RedrawRequested);
    }

    pub(crate) fn request_animation_frame(&self) {
        self.redraw.request_animation_frame();
    }

    pub(crate) fn size(&self) -> (u32, u32) {
        if self.is_lost() {
            return (0, 0);
        }
        let width = unsafe { vk::gtk_widget_get_allocated_width(self.widget) };
        let height = unsafe { vk::gtk_widget_get_allocated_height(self.widget) };
        let scale = unsafe { gtk_widget_get_scale_factor(self.widget) }.max(1);
        (
            width.max(0).saturating_mul(scale) as u32,
            height.max(0).saturating_mul(scale) as u32,
        )
    }

    pub(super) fn is_lost(&self) -> bool {
        self.owner.widget.lost.get() || unsafe { gtk_widget_get_window(self.widget) }.is_null()
    }
}

pub(super) struct Swapchain {
    _surface: Rc<SurfaceOwner>,
    handle: u64,
    device: Rc<NativeDevice>,
    images: Vec<u64>,
    views: Vec<u64>,
    finished: Vec<u64>,
    size: (u32, u32),
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            let d = &self.device;
            call!(d.instance, DeviceWaitIdle, d.handle);
            for &view in &self.views {
                call!(d.instance, DestroyImageView, d.handle, view, ptr::null());
            }
            for &sem in &self.finished {
                call!(d.instance, DestroySemaphore, d.handle, sem, ptr::null());
            }
            call!(
                d.instance,
                DestroySwapchainKHR,
                d.handle,
                self.handle,
                ptr::null()
            );
        }
    }
}

impl Context {
    pub(crate) fn with_surface(surface: &Surface) -> Result<Rc<Self>> {
        Self::create(&surface.instance, Some(surface.handle))
    }

    /// Whether this submission consumes the acquisition semaphore.
    ///
    /// A binary semaphore's signal can only be consumed once per acquisition.
    pub(super) fn take_acquisition_wait(&self) -> bool {
        self.acquired.borrow().is_some() && !self.acquisition_waited.replace(true)
    }

    pub(crate) fn formats(&self, surface: &Surface) -> Result<Vec<TextureFormat>> {
        unsafe {
            let d = &self.device;
            let mut count = 0;
            check(call!(
                d.instance,
                GetPhysicalDeviceSurfaceFormatsKHR,
                d.physical,
                surface.handle,
                &mut count,
                ptr::null_mut()
            ))?;
            let mut formats = vec![vk::SurfaceFormatKHR::default(); count as usize];
            check(call!(
                d.instance,
                GetPhysicalDeviceSurfaceFormatsKHR,
                d.physical,
                surface.handle,
                &mut count,
                formats.as_mut_ptr()
            ))?;
            if formats.len() == 1 && formats[0].format == 0 {
                return Ok(vec![TextureFormat::Bgra8UnormSrgb]);
            }
            Ok(formats
                .iter()
                .filter(|f| f.color_space == 0)
                .filter_map(|f| from_format(f.format))
                .collect())
        }
    }

    pub(crate) fn configure(&self, surface: &Surface, config: &SurfaceConfiguration) -> Result<()> {
        if surface.is_lost() {
            return Err(Error(
                "GTK presentation surface is no longer available".into(),
            ));
        }
        unsafe {
            let d = &self.device;
            require(
                Rc::ptr_eq(&surface.instance, &d.instance),
                "Surface belongs to another Vulkan instance",
            );
            require(
                self.acquired.borrow().is_none(),
                "Present the acquired frame before reconfiguring the surface",
            );
            for frame in self.frames.borrow_mut().iter_mut() {
                self.wait(frame, u64::MAX)?;
            }
            check(call!(d.instance, DeviceWaitIdle, d.handle))?;
            self.surface_status.set(0);
            let mut caps = vk::SurfaceCapabilitiesKHR::default();
            check(call!(
                d.instance,
                GetPhysicalDeviceSurfaceCapabilitiesKHR,
                d.physical,
                surface.handle,
                &mut caps
            ))?;
            let extent = if caps.current_extent.width != u32::MAX {
                caps.current_extent
            } else {
                vk::Extent2D {
                    width: config
                        .width
                        .clamp(caps.min_image_extent.width, caps.max_image_extent.width),
                    height: config
                        .height
                        .clamp(caps.min_image_extent.height, caps.max_image_extent.height),
                }
            };
            if extent.width != config.width || extent.height != config.height {
                return Err(Error(
                    "Surface extent differs from window extent; retry after resize".into(),
                ));
            }
            let count = if caps.max_image_count == 0 {
                caps.min_image_count.max(3)
            } else {
                caps.min_image_count.max(3).min(caps.max_image_count)
            };
            let old = surface.swap.borrow().as_ref().map_or(0, |swap| swap.handle);
            let info = vk::SwapchainCreateInfoKHR {
                s_type: 1000001000,
                surface: surface.handle,
                min_image_count: count,
                image_format: format(config.format),
                image_color_space: 0,
                image_extent: extent,
                image_array_layers: 1,
                image_usage: 16,
                pre_transform: caps.current_transform,
                composite_alpha: 1 << caps.supported_composite_alpha.trailing_zeros(),
                present_mode: 2,
                clipped: 1,
                old_swapchain: old,
                ..Default::default()
            };
            let mut swap = Swapchain {
                _surface: surface.owner.clone(),
                handle: 0,
                device: d.clone(),
                images: Vec::new(),
                views: Vec::new(),
                finished: Vec::new(),
                size: (extent.width, extent.height),
            };
            check(call!(
                d.instance,
                CreateSwapchainKHR,
                d.handle,
                &info,
                ptr::null(),
                &mut swap.handle
            ))?;
            let mut count = 0;
            check(call!(
                d.instance,
                GetSwapchainImagesKHR,
                d.handle,
                swap.handle,
                &mut count,
                ptr::null_mut()
            ))?;
            swap.images.resize(count as usize, 0);
            check(call!(
                d.instance,
                GetSwapchainImagesKHR,
                d.handle,
                swap.handle,
                &mut count,
                swap.images.as_mut_ptr()
            ))?;
            for &image in &swap.images {
                swap.views.push(self.view(image, config.format, 1, 1)?);
                let info = vk::SemaphoreCreateInfo {
                    s_type: 9,
                    ..Default::default()
                };
                let mut sem = 0;
                check(call!(
                    d.instance,
                    CreateSemaphore,
                    d.handle,
                    &info,
                    ptr::null(),
                    &mut sem
                ))?;
                swap.finished.push(sem);
            }
            *surface.swap.borrow_mut() = Some(Rc::new(swap));
            Ok(())
        }
    }

    pub(crate) fn acquire(
        &self,
        surface: &Surface,
    ) -> std::result::Result<(Texture, bool), CurrentSurfaceTexture> {
        // GTK may have destroyed the native surface while the application still
        // retains its renderer. Never pass its stale Wayland/X11 handle to WSI.
        if surface.is_lost() {
            return Err(CurrentSurfaceTexture::Lost);
        }
        if !surface.redraw.ready.get() {
            return Err(CurrentSurfaceTexture::Occluded);
        }
        match self.surface_status.replace(0) {
            -1000001004 | 1000001003 => return Err(CurrentSurfaceTexture::Outdated),
            -1000000000 => return Err(CurrentSurfaceTexture::Lost),
            _ => {}
        }
        if self.acquired.borrow().is_some() {
            return Err(CurrentSurfaceTexture::Validation);
        }
        let mut frames = self.frames.borrow_mut();
        let frame = &mut frames[self.slot.get()];
        // Yield to GTK while the GPU is busy; the caller retries on a later frame tick.
        if !self
            .wait(frame, 0)
            .map_err(|_| CurrentSurfaceTexture::Lost)?
        {
            return Err(CurrentSurfaceTexture::Timeout);
        }
        let swap = surface
            .swap
            .borrow()
            .clone()
            .ok_or(CurrentSurfaceTexture::Validation)?;
        // Wayland has no fixed surface extent, so the driver need not report
        // OUT_OF_DATE when GTK resizes the content or changes its scale.
        if swap.size != surface.size() {
            return Err(CurrentSurfaceTexture::Outdated);
        }
        let mut index = 0;
        let result = unsafe {
            call!(
                self.device.instance,
                AcquireNextImageKHR,
                self.device.handle,
                swap.handle,
                0,
                frame.available,
                0,
                &mut index
            )
        };
        match result {
            0 | 1000001003 => {}
            -1000001004 => return Err(CurrentSurfaceTexture::Outdated),
            -1000000000 => return Err(CurrentSurfaceTexture::Lost),
            -4 => return Err(CurrentSurfaceTexture::Validation),
            1 | 2 => return Err(CurrentSurfaceTexture::Timeout),
            _ => return Err(CurrentSurfaceTexture::Validation),
        }
        *self.acquired.borrow_mut() = Some((swap.clone(), index as usize));
        self.acquisition_waited.set(false);
        Ok((
            Texture {
                image: swap.images[index as usize],
                view: swap.views[index as usize],
                memory: None,
                swap: Some(swap),
                device: self.device.clone(),
                layout: Cell::new(0),
                mips: 1,
                layers: 1,
                depth: false,
                index: index as usize,
            },
            result != 0,
        ))
    }

    pub(crate) fn present(&self, surface: &Surface, texture: &Texture) -> Result<()> {
        require(
            self.acquisition_waited.get(),
            "Present requires a submission after acquisition",
        );
        let (swap, index) = self
            .acquired
            .borrow_mut()
            .take()
            .ok_or_else(|| Error("No acquired frame".into()))?;
        require(
            texture.index == index && texture.swap.as_ref().is_some_and(|s| Rc::ptr_eq(s, &swap)),
            "Wrong surface texture",
        );
        if surface.is_lost() {
            self.acquisition_waited.set(false);
            self.surface_status.set(-1000000000);
            self.slot.set((self.slot.get() + 1) % 2);
            return Ok(());
        }
        // Signal only after all submissions for this image. Signaling on every
        // submit would reuse an unconsumed binary semaphore before presentation.
        let ready = vk::SubmitInfo {
            s_type: 4,
            signal_semaphore_count: 1,
            p_signal_semaphores: &swap.finished[index],
            ..Default::default()
        };
        unsafe {
            check(call!(
                self.device.instance,
                QueueSubmit,
                self.device.queue,
                1,
                &ready,
                0
            ))?;
        }
        self.acquisition_waited.set(false);
        let index32 = index as u32;
        let info = vk::PresentInfoKHR {
            s_type: 1000001001,
            wait_semaphore_count: 1,
            p_wait_semaphores: &swap.finished[index],
            swapchain_count: 1,
            p_swapchains: &swap.handle,
            p_image_indices: &index32,
            ..Default::default()
        };
        let result = unsafe {
            call!(
                self.device.instance,
                QueuePresentKHR,
                self.device.queue,
                &info
            )
        };
        self.slot.set((self.slot.get() + 1) % 2);
        if matches!(result, 1000001003 | -1000001004 | -1000000000) {
            self.surface_status.set(result);
            Ok(())
        } else {
            check(result)
        }
    }
}
