/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! OpenSSL TLS implementation for Linux and other Unix platforms.
//!
//! Supports both OpenSSL 1.1.x (Ubuntu 18.04/20.04, RHEL 7/8) and
//! OpenSSL 3.x (Ubuntu 22.04+, Debian 12, Fedora, Arch). The API surface
//! used (SSL_CTX_new, SSL_connect, BIO_meth_new, etc.) is identical in both.

use std::ffi::{CString, c_char, c_int, c_long, c_void};
use std::io::{self, Read, Write};
use std::slice;

use crate::{Error, HandshakeError};

// MARK: OpenSSL FFI
// All functions used are identical in OpenSSL 1.1.1 and 3.x

#[link(name = "ssl")]
#[link(name = "crypto")]
unsafe extern "C" {
    // SSL_CTX
    fn SSL_CTX_new(method: *const c_void) -> *mut c_void;
    fn SSL_CTX_free(ctx: *mut c_void);
    fn SSL_CTX_set_verify(ctx: *mut c_void, mode: c_int, callback: *const c_void);
    fn SSL_CTX_set_default_verify_paths(ctx: *mut c_void) -> c_int;
    fn SSL_CTX_ctrl(ctx: *mut c_void, cmd: c_int, larg: c_long, parg: *mut c_void) -> c_long;
    fn TLS_client_method() -> *const c_void;

    // SSL
    fn SSL_new(ctx: *mut c_void) -> *mut c_void;
    fn SSL_free(ssl: *mut c_void);
    fn SSL_set_bio(ssl: *mut c_void, rbio: *mut c_void, wbio: *mut c_void);
    fn SSL_set1_host(ssl: *mut c_void, hostname: *const c_char) -> c_int;
    fn SSL_connect(ssl: *mut c_void) -> c_int;
    fn SSL_read(ssl: *mut c_void, buf: *mut c_void, num: c_int) -> c_int;
    fn SSL_write(ssl: *mut c_void, buf: *const c_void, num: c_int) -> c_int;
    fn SSL_shutdown(ssl: *mut c_void) -> c_int;
    fn SSL_get_error(ssl: *const c_void, ret: c_int) -> c_int;
    fn SSL_ctrl(ssl: *mut c_void, cmd: c_int, larg: c_long, parg: *mut c_void) -> c_long;

    // BIO methods
    fn BIO_meth_new(type_: c_int, name: *const c_char) -> *mut c_void;
    fn BIO_meth_free(biom: *mut c_void);
    fn BIO_meth_set_read(
        biom: *mut c_void,
        read: unsafe extern "C" fn(*mut c_void, *mut c_char, c_int) -> c_int,
    ) -> c_int;
    fn BIO_meth_set_write(
        biom: *mut c_void,
        write: unsafe extern "C" fn(*mut c_void, *const c_char, c_int) -> c_int,
    ) -> c_int;
    fn BIO_meth_set_ctrl(
        biom: *mut c_void,
        ctrl: unsafe extern "C" fn(*mut c_void, c_int, c_long, *mut c_void) -> c_long,
    ) -> c_int;
    fn BIO_meth_set_create(
        biom: *mut c_void,
        create: unsafe extern "C" fn(*mut c_void) -> c_int,
    ) -> c_int;
    fn BIO_meth_set_destroy(
        biom: *mut c_void,
        destroy: unsafe extern "C" fn(*mut c_void) -> c_int,
    ) -> c_int;

    // BIO instance
    fn BIO_new(biom: *const c_void) -> *mut c_void;
    fn BIO_set_data(bio: *mut c_void, data: *mut c_void);
    fn BIO_get_data(bio: *const c_void) -> *mut c_void;
    fn BIO_set_init(bio: *mut c_void, init: c_int);
}

// SSL_CTX_ctrl commands for protocol version
const SSL_CTRL_SET_MIN_PROTO_VERSION: c_int = 123;
const SSL_CTRL_SET_MAX_PROTO_VERSION: c_int = 124;
const TLS1_2_VERSION: c_long = 0x0303;
const TLS1_3_VERSION: c_long = 0x0304;

// SSL_CTX_set_verify mode
const SSL_VERIFY_PEER: c_int = 0x01;

// SSL_get_error return codes
const SSL_ERROR_NONE: c_int = 0;
const SSL_ERROR_ZERO_RETURN: c_int = 6;
const SSL_ERROR_WANT_READ: c_int = 2;
const SSL_ERROR_WANT_WRITE: c_int = 3;

// SSL_ctrl command for SNI (SSL_set_tlsext_host_name macro equivalent)
const SSL_CTRL_SET_TLSEXT_HOSTNAME: c_int = 55;
const TLSEXT_NAMETYPE_HOST_NAME: c_long = 0;

// Custom BIO type: SOURCE_SINK bit (0x0400) | custom ID
const CUSTOM_BIO_TYPE: c_int = 0x0400 | 100;

// BIO_ctrl cmd
const BIO_CTRL_FLUSH: c_int = 11;

// MARK: BIO callbacks
// Type-erased stream wrapper stored in the BIO's data pointer
struct IoFuncs {
    stream_ptr: *mut c_void,
    read_fn: unsafe fn(*mut c_void, *mut u8, usize) -> io::Result<usize>,
    write_fn: unsafe fn(*mut c_void, *const u8, usize) -> io::Result<usize>,
}

unsafe extern "C" fn bio_read(bio: *mut c_void, buf: *mut c_char, len: c_int) -> c_int {
    if len <= 0 {
        return 0;
    }
    let io = unsafe { &mut *(BIO_get_data(bio) as *mut IoFuncs) };
    match unsafe { (io.read_fn)(io.stream_ptr, buf.cast::<u8>(), len as usize) } {
        Ok(0) => 0,
        Ok(n) => n as c_int,
        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => -1,
        Err(_) => -1,
    }
}

unsafe extern "C" fn bio_write(bio: *mut c_void, buf: *const c_char, len: c_int) -> c_int {
    if len <= 0 {
        return 0;
    }
    let io = unsafe { &mut *(BIO_get_data(bio) as *mut IoFuncs) };
    match unsafe { (io.write_fn)(io.stream_ptr, buf.cast::<u8>(), len as usize) } {
        Ok(n) => n as c_int,
        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => -1,
        Err(_) => -1,
    }
}

unsafe extern "C" fn bio_ctrl(
    _bio: *mut c_void,
    cmd: c_int,
    _larg: c_long,
    _parg: *mut c_void,
) -> c_long {
    if cmd == BIO_CTRL_FLUSH { 1 } else { 0 }
}

unsafe extern "C" fn bio_create(bio: *mut c_void) -> c_int {
    unsafe { BIO_set_init(bio, 1) };
    1
}

unsafe extern "C" fn bio_destroy(_bio: *mut c_void) -> c_int {
    // IoFuncs is owned by TlsStream, do not free here
    1
}

unsafe fn do_read<S: Read>(ptr: *mut c_void, buf: *mut u8, len: usize) -> io::Result<usize> {
    unsafe { (*(ptr as *mut S)).read(slice::from_raw_parts_mut(buf, len)) }
}

unsafe fn do_write<S: Write>(ptr: *mut c_void, buf: *const u8, len: usize) -> io::Result<usize> {
    unsafe { (*(ptr as *mut S)).write(slice::from_raw_parts(buf, len)) }
}

// Create the custom BIO method table (shared, created once)
fn make_bio_method() -> *mut c_void {
    let name = c"rust-stream";
    unsafe {
        let m = BIO_meth_new(CUSTOM_BIO_TYPE, name.as_ptr());
        BIO_meth_set_read(m, bio_read);
        BIO_meth_set_write(m, bio_write);
        BIO_meth_set_ctrl(m, bio_ctrl);
        BIO_meth_set_create(m, bio_create);
        BIO_meth_set_destroy(m, bio_destroy);
        m
    }
}

struct BioMethodPtr(*mut c_void);
// SAFETY: BIO_METHOD is immutable after creation and OpenSSL guarantees thread safety for it
unsafe impl Send for BioMethodPtr {}
unsafe impl Sync for BioMethodPtr {}

impl Drop for BioMethodPtr {
    fn drop(&mut self) {
        unsafe { BIO_meth_free(self.0) };
    }
}

static BIO_METHOD: std::sync::LazyLock<BioMethodPtr> =
    std::sync::LazyLock::new(|| BioMethodPtr(make_bio_method()));

// MARK: TlsConnector
/// A TLS connector using OpenSSL
pub struct TlsConnector {
    ctx: *mut c_void,
}

// SAFETY: SSL_CTX is thread-safe for concurrent use after initialization
unsafe impl Send for TlsConnector {}
unsafe impl Sync for TlsConnector {}

impl TlsConnector {
    /// Create a new TLS connector
    pub fn new() -> Result<Self, Error> {
        let ctx = unsafe { SSL_CTX_new(TLS_client_method()) };
        if ctx.is_null() {
            return Err(Error("Failed to create SSL_CTX".to_string()));
        }

        unsafe {
            // Set TLS 1.2 as minimum and TLS 1.3 as maximum
            SSL_CTX_ctrl(
                ctx,
                SSL_CTRL_SET_MIN_PROTO_VERSION,
                TLS1_2_VERSION,
                std::ptr::null_mut(),
            );
            SSL_CTX_ctrl(
                ctx,
                SSL_CTRL_SET_MAX_PROTO_VERSION,
                TLS1_3_VERSION,
                std::ptr::null_mut(),
            );

            // Enable certificate verification using system/default CA store
            SSL_CTX_set_verify(ctx, SSL_VERIFY_PEER, std::ptr::null());
            SSL_CTX_set_default_verify_paths(ctx);
        }

        Ok(Self { ctx })
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
        let ssl = unsafe { SSL_new(self.ctx) };
        if ssl.is_null() {
            return Err(Error("Failed to create SSL".to_string()));
        }

        // Set SNI hostname (equivalent to SSL_set_tlsext_host_name macro)
        let domain_c =
            CString::new(domain).map_err(|_| Error("Invalid domain name".to_string()))?;
        unsafe {
            SSL_ctrl(
                ssl,
                SSL_CTRL_SET_TLSEXT_HOSTNAME,
                TLSEXT_NAMETYPE_HOST_NAME,
                domain_c.as_ptr() as *mut c_void,
            );
            // Enable hostname verification against the certificate
            if SSL_set1_host(ssl, domain_c.as_ptr()) != 1 {
                SSL_free(ssl);
                return Err(Error("Failed to set hostname for verification".to_string()));
            }
        }

        // Box the stream for stable heap address
        let stream = Box::new(stream);
        let io = Box::new(IoFuncs {
            stream_ptr: stream.as_ref() as *const S as *mut c_void,
            read_fn: do_read::<S>,
            write_fn: do_write::<S>,
        });

        // Create and configure the custom BIO
        let bio = unsafe { BIO_new(BIO_METHOD.0) };
        if bio.is_null() {
            unsafe { SSL_free(ssl) };
            return Err(Error("Failed to create BIO".to_string()));
        }
        unsafe {
            BIO_set_data(bio, io.as_ref() as *const IoFuncs as *mut c_void);
            // Both read and write use the same BIO
            SSL_set_bio(ssl, bio, bio);
        }

        // Perform handshake
        loop {
            let ret = unsafe { SSL_connect(ssl) };
            if ret == 1 {
                break;
            }
            let err = unsafe { SSL_get_error(ssl, ret) };
            match err {
                SSL_ERROR_WANT_READ | SSL_ERROR_WANT_WRITE => continue,
                _ => {
                    unsafe { SSL_free(ssl) };
                    return Err(Error(format!("TLS handshake failed (SSL_get_error={err})")));
                }
            }
        }

        Ok(TlsStream {
            ssl,
            _io: io,
            stream,
        })
    }
}

impl Drop for TlsConnector {
    fn drop(&mut self) {
        unsafe { SSL_CTX_free(self.ctx) };
    }
}

// MARK: TlsStream
/// A TLS stream backed by OpenSSL
pub struct TlsStream<S> {
    ssl: *mut c_void,
    _io: Box<IoFuncs>,
    stream: Box<S>,
}

// SAFETY: TlsStream owns ssl and stream, both are safe to send across threads
unsafe impl<S: Send> Send for TlsStream<S> {}

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
        let ret = unsafe {
            SSL_read(
                self.ssl,
                buf.as_mut_ptr() as *mut c_void,
                buf.len() as c_int,
            )
        };
        if ret > 0 {
            return Ok(ret as usize);
        }
        let err = unsafe { SSL_get_error(self.ssl, ret) };
        match err {
            SSL_ERROR_ZERO_RETURN => Ok(0),
            SSL_ERROR_NONE => Ok(0),
            SSL_ERROR_WANT_READ | SSL_ERROR_WANT_WRITE => {
                Err(io::Error::new(io::ErrorKind::WouldBlock, "TLS would block"))
            }
            _ => Err(io::Error::other(format!("SSL_read error: {err}"))),
        }
    }
}

impl<S: Read + Write> Write for TlsStream<S> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        let ret = unsafe { SSL_write(self.ssl, buf.as_ptr() as *const c_void, buf.len() as c_int) };
        if ret > 0 {
            return Ok(ret as usize);
        }
        let err = unsafe { SSL_get_error(self.ssl, ret) };
        match err {
            SSL_ERROR_WANT_READ | SSL_ERROR_WANT_WRITE => {
                Err(io::Error::new(io::ErrorKind::WouldBlock, "TLS would block"))
            }
            _ => Err(io::Error::other(format!("SSL_write error: {err}"))),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<S> Drop for TlsStream<S> {
    fn drop(&mut self) {
        unsafe {
            SSL_shutdown(self.ssl);
            SSL_free(self.ssl); // also frees the BIO
        }
    }
}
