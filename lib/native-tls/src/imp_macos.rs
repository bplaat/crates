/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! macOS TLS implementation using Security.framework SecureTransport

use std::ffi::c_void;
use std::io::{self, Read, Write};
use std::os::raw::{c_char, c_int};
use std::slice;

use crate::{Error, HandshakeError};

// MARK: FFI types and constants
type SSLContextRef = *mut c_void;
type SSLConnectionRef = *const c_void;
type SecTrustRef = *mut c_void;
type CFErrorRef = *mut c_void;
type OSStatus = i32;
type SSLReadFunc = unsafe extern "C" fn(SSLConnectionRef, *mut c_void, *mut usize) -> OSStatus;
type SSLWriteFunc = unsafe extern "C" fn(SSLConnectionRef, *const c_void, *mut usize) -> OSStatus;

const SSL_CLIENT_SIDE: u32 = 1;
const SSL_STREAM_TYPE: u32 = 0;
// kSSLSessionOptionBreakOnServerAuth = 0: pause handshake after server cert received for manual trust eval
const SSL_OPT_BREAK_ON_SERVER_AUTH: i32 = 0;
// kTLSProtocol12 = 8: enforce TLS 1.2 as both minimum and maximum version (SSLProtocol is c_int)
const K_TLS_PROTOCOL_12: c_int = 8;
const NO_ERR: OSStatus = 0;
const ERR_SSL_WOULD_BLOCK: OSStatus = -9803;
const ERR_SSL_CLOSED_GRACEFUL: OSStatus = -9805;
const ERR_SSL_CLOSED_ABORT: OSStatus = -9806;
const ERR_SSL_CLOSED_NO_NOTIFY: OSStatus = -9816;
// errSSLPeerAuthCompleted: returned when break-on-server-auth is set and server cert is ready to evaluate
const ERR_SSL_PEER_AUTH_COMPLETED: OSStatus = -9841;
const IO_ERR: OSStatus = -36;

#[link(name = "Security", kind = "framework")]
unsafe extern "C" {
    fn SSLCreateContext(
        alloc: *const c_void,
        protocol_side: u32,
        connection_type: u32,
    ) -> SSLContextRef;
    fn SSLSetSessionOption(ctx: SSLContextRef, option: i32, value: u8) -> OSStatus;
    fn SSLSetIOFuncs(ctx: SSLContextRef, read: SSLReadFunc, write: SSLWriteFunc) -> OSStatus;
    fn SSLSetConnection(ctx: SSLContextRef, connection: SSLConnectionRef) -> OSStatus;
    fn SSLSetProtocolVersionMin(ctx: SSLContextRef, min_ver: c_int) -> OSStatus;
    fn SSLSetProtocolVersionMax(ctx: SSLContextRef, max_ver: c_int) -> OSStatus;
    fn SSLSetPeerDomainName(ctx: SSLContextRef, name: *const c_char, len: usize) -> OSStatus;
    fn SSLHandshake(ctx: SSLContextRef) -> OSStatus;
    fn SSLCopyPeerTrust(ctx: SSLContextRef, trust: *mut SecTrustRef) -> OSStatus;
    fn SecTrustEvaluateWithError(trust: SecTrustRef, error: *mut CFErrorRef) -> u8;
    fn SSLRead(
        ctx: SSLContextRef,
        data: *mut c_void,
        data_len: usize,
        processed: *mut usize,
    ) -> OSStatus;
    fn SSLWrite(
        ctx: SSLContextRef,
        data: *const c_void,
        data_len: usize,
        processed: *mut usize,
    ) -> OSStatus;
    fn SSLClose(ctx: SSLContextRef) -> OSStatus;
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFRelease(cf: *const c_void);
}

// MARK: IoFuncs
// Type-erased I/O wrapper passed as the SSL connection reference
struct IoFuncs {
    stream_ptr: *mut c_void,
    read_fn: unsafe fn(*mut c_void, *mut u8, usize) -> io::Result<usize>,
    write_fn: unsafe fn(*mut c_void, *const u8, usize) -> io::Result<usize>,
}

unsafe extern "C" fn ssl_read_cb(
    conn: SSLConnectionRef,
    data: *mut c_void,
    data_len: *mut usize,
) -> OSStatus {
    if data.is_null() || data_len.is_null() || conn.is_null() {
        return IO_ERR;
    }
    let io = unsafe { &mut *(conn as *mut IoFuncs) };
    let total = unsafe { *data_len };
    let buf = unsafe { slice::from_raw_parts_mut(data as *mut u8, total) };
    let mut got = 0usize;
    // SecureTransport requires all `total` bytes to be read before returning noErr
    while got < total {
        match unsafe { (io.read_fn)(io.stream_ptr, buf.as_mut_ptr().add(got), total - got) } {
            Ok(0) => {
                unsafe { *data_len = got };
                return ERR_SSL_CLOSED_NO_NOTIFY;
            }
            Ok(n) => got += n,
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                unsafe { *data_len = got };
                return ERR_SSL_WOULD_BLOCK;
            }
            Err(_) => {
                unsafe { *data_len = got };
                return IO_ERR;
            }
        }
    }
    unsafe { *data_len = got };
    NO_ERR
}

unsafe extern "C" fn ssl_write_cb(
    conn: SSLConnectionRef,
    data: *const c_void,
    data_len: *mut usize,
) -> OSStatus {
    if data.is_null() || data_len.is_null() || conn.is_null() {
        return IO_ERR;
    }
    let io = unsafe { &mut *(conn as *mut IoFuncs) };
    let total = unsafe { *data_len };
    let buf = unsafe { slice::from_raw_parts(data as *const u8, total) };
    let mut sent = 0usize;
    // SecureTransport requires all `total` bytes to be written before returning noErr
    while sent < total {
        match unsafe { (io.write_fn)(io.stream_ptr, buf.as_ptr().add(sent), total - sent) } {
            Ok(0) => {
                unsafe { *data_len = sent };
                return ERR_SSL_CLOSED_NO_NOTIFY;
            }
            Ok(n) => sent += n,
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                unsafe { *data_len = sent };
                return ERR_SSL_WOULD_BLOCK;
            }
            Err(_) => {
                unsafe { *data_len = sent };
                return IO_ERR;
            }
        }
    }
    unsafe { *data_len = sent };
    NO_ERR
}

unsafe fn do_read<S: Read>(ptr: *mut c_void, buf: *mut u8, len: usize) -> io::Result<usize> {
    let stream = unsafe { &mut *(ptr as *mut S) };
    stream.read(unsafe { slice::from_raw_parts_mut(buf, len) })
}

unsafe fn do_write<S: Write>(ptr: *mut c_void, buf: *const u8, len: usize) -> io::Result<usize> {
    let stream = unsafe { &mut *(ptr as *mut S) };
    stream.write(unsafe { slice::from_raw_parts(buf, len) })
}

// MARK: TlsConnector
/// A TLS connector using SecureTransport
pub struct TlsConnector;

impl TlsConnector {
    /// Create a new TLS connector
    pub fn new() -> Result<Self, Error> {
        Ok(Self)
    }

    /// Perform a TLS handshake over the given stream
    pub fn connect<S: Read + Write>(
        &self,
        domain: &str,
        stream: S,
    ) -> Result<TlsStream<S>, HandshakeError<S>> {
        self.connect_inner(domain, stream)
            .map_err(HandshakeError::new)
    }

    fn connect_inner<S: Read + Write>(
        &self,
        domain: &str,
        stream: S,
    ) -> Result<TlsStream<S>, Error> {
        // Create SSL context
        let ctx = unsafe { SSLCreateContext(std::ptr::null(), SSL_CLIENT_SIDE, SSL_STREAM_TYPE) };
        if ctx.is_null() {
            return Err(Error("Failed to create SSL context".to_string()));
        }

        // Break handshake after server cert is received so we can do manual trust evaluation.
        // Without this, SSLHandshake tries implicit validation which is broken on macOS 26+.
        let status = unsafe { SSLSetSessionOption(ctx, SSL_OPT_BREAK_ON_SERVER_AUTH, 1) };
        if status != NO_ERR {
            unsafe { CFRelease(ctx as *const c_void) };
            return Err(Error(format!("SSLSetSessionOption failed: {status}")));
        }

        // Restrict to TLS 1.2 only
        let status = unsafe { SSLSetProtocolVersionMin(ctx, K_TLS_PROTOCOL_12) };
        if status != NO_ERR {
            unsafe { CFRelease(ctx as *const c_void) };
            return Err(Error(format!("SSLSetProtocolVersionMin failed: {status}")));
        }
        let status = unsafe { SSLSetProtocolVersionMax(ctx, K_TLS_PROTOCOL_12) };
        if status != NO_ERR {
            unsafe { CFRelease(ctx as *const c_void) };
            return Err(Error(format!("SSLSetProtocolVersionMax failed: {status}")));
        }

        // Store stream in a Box to ensure stable heap address
        let stream = Box::new(stream);
        let io = Box::new(IoFuncs {
            stream_ptr: stream.as_ref() as *const S as *mut c_void,
            read_fn: do_read::<S>,
            write_fn: do_write::<S>,
        });

        // Register I/O callbacks - IOFuncs must be set before Connection
        let status = unsafe { SSLSetIOFuncs(ctx, ssl_read_cb, ssl_write_cb) };
        if status != NO_ERR {
            unsafe { CFRelease(ctx as *const c_void) };
            return Err(Error(format!("SSLSetIOFuncs failed: {status}")));
        }
        let status =
            unsafe { SSLSetConnection(ctx, io.as_ref() as *const IoFuncs as SSLConnectionRef) };
        if status != NO_ERR {
            unsafe { CFRelease(ctx as *const c_void) };
            return Err(Error(format!("SSLSetConnection failed: {status}")));
        }

        // Set SNI
        let domain_bytes = domain.as_bytes();
        let status = unsafe {
            SSLSetPeerDomainName(
                ctx,
                domain_bytes.as_ptr() as *const c_char,
                domain_bytes.len(),
            )
        };
        if status != NO_ERR {
            unsafe { CFRelease(ctx as *const c_void) };
            return Err(Error(format!("SSLSetPeerDomainName failed: {status}")));
        }

        // Perform handshake loop; handle trust evaluation break-point
        loop {
            let status = unsafe { SSLHandshake(ctx) };
            match status {
                NO_ERR => break,
                ERR_SSL_WOULD_BLOCK => continue,
                ERR_SSL_PEER_AUTH_COMPLETED => {
                    // Server cert is ready; evaluate trust manually using the modern API
                    let mut trust: SecTrustRef = std::ptr::null_mut();
                    let st = unsafe { SSLCopyPeerTrust(ctx, &mut trust) };
                    if st != NO_ERR || trust.is_null() {
                        unsafe {
                            SSLClose(ctx);
                            CFRelease(ctx as *const c_void);
                        }
                        return Err(Error(format!("SSLCopyPeerTrust failed: {st}")));
                    }
                    let mut cf_error: CFErrorRef = std::ptr::null_mut();
                    let trusted = unsafe { SecTrustEvaluateWithError(trust, &mut cf_error) };
                    unsafe { CFRelease(trust) };
                    if !cf_error.is_null() {
                        unsafe { CFRelease(cf_error) };
                    }
                    if trusted == 0 {
                        unsafe {
                            SSLClose(ctx);
                            CFRelease(ctx as *const c_void);
                        }
                        return Err(Error("Certificate trust validation failed".to_string()));
                    }
                    // Trust OK; resume handshake
                    continue;
                }
                _ => {
                    unsafe {
                        SSLClose(ctx);
                        CFRelease(ctx as *const c_void);
                    }
                    return Err(Error(format!("TLS handshake failed: {status}")));
                }
            }
        }

        Ok(TlsStream { ctx, io, stream })
    }
}

// MARK: TlsStream
/// A TLS stream backed by SecureTransport
pub struct TlsStream<S> {
    ctx: SSLContextRef,
    // Kept alive as the SSL connection reference pointer; must not be moved or dropped before ctx
    #[expect(dead_code)]
    io: Box<IoFuncs>,
    stream: Box<S>,
}

impl<S> TlsStream<S> {
    /// Returns a reference to the underlying stream
    pub fn get_ref(&self) -> &S {
        &self.stream
    }

    /// Returns a mutable reference to the underlying stream
    pub fn get_mut(&mut self) -> &mut S {
        &mut self.stream
    }
}

impl<S: Read + Write> Read for TlsStream<S> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        let mut processed = 0usize;
        let status = unsafe {
            SSLRead(
                self.ctx,
                buf.as_mut_ptr() as *mut c_void,
                buf.len(),
                &mut processed,
            )
        };
        // SSLRead may return an error and data simultaneously; prioritize data
        if processed > 0 {
            return Ok(processed);
        }
        match status {
            NO_ERR => Ok(processed),
            ERR_SSL_CLOSED_GRACEFUL | ERR_SSL_CLOSED_ABORT | ERR_SSL_CLOSED_NO_NOTIFY => Ok(0),
            ERR_SSL_WOULD_BLOCK => {
                Err(io::Error::new(io::ErrorKind::WouldBlock, "TLS would block"))
            }
            _ => Err(io::Error::other(format!("SSLRead error: {status}"))),
        }
    }
}

impl<S: Read + Write> Write for TlsStream<S> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut processed = 0usize;
        let status = unsafe {
            SSLWrite(
                self.ctx,
                buf.as_ptr() as *const c_void,
                buf.len(),
                &mut processed,
            )
        };
        match status {
            NO_ERR => Ok(processed),
            ERR_SSL_WOULD_BLOCK => {
                if processed > 0 {
                    Ok(processed)
                } else {
                    Err(io::Error::new(io::ErrorKind::WouldBlock, "TLS would block"))
                }
            }
            _ => Err(io::Error::other(format!("SSLWrite error: {status}"))),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<S> Drop for TlsStream<S> {
    fn drop(&mut self) {
        unsafe {
            SSLClose(self.ctx);
            CFRelease(self.ctx as *const c_void);
        }
    }
}
