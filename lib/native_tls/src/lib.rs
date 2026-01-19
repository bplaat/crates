/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [native_tls](https://crates.io/crates/native_tls) crate

use std::io::{self, Read, Write};
use std::net::TcpStream;

/// A TLS connector
pub struct TlsConnector {}

impl TlsConnector {
    /// Creates a new `TlsConnector`.
    pub fn new() -> io::Result<Self> {
        Ok(TlsConnector {})
    }

    /// Connects to a TLS server.
    pub fn connect(&self, domain: &str, stream: TcpStream) -> io::Result<TlsStream> {
        #[cfg(target_os = "macos")]
        {
            use core_foundation_sys::base::CFRelease;
            use security_framework_sys::secure_transport::{
                SSLClose, SSLCreateContext, SSLHandshake, SSLSetConnection, SSLSetIOFuncs,
                SSLSetPeerDomainName, errSSLWouldBlock, kSSLClientSide, kSSLStreamType,
            };

            let context =
                unsafe { SSLCreateContext(std::ptr::null(), kSSLClientSide, kSSLStreamType) };
            if context.is_null() {
                return Err(io::Error::other("Failed to create SSL context"));
            }

            stream.set_nonblocking(false)?;

            unsafe {
                SSLSetIOFuncs(context, read_func, write_func);
                SSLSetConnection(context, Box::into_raw(Box::new(stream)) as *mut _);

                let domain_cstring = std::ffi::CString::new(domain)
                    .map_err(|_| io::Error::other("Invalid domain name"))?;
                SSLSetPeerDomainName(context, domain_cstring.as_ptr() as *const _, domain.len());

                let mut status;
                loop {
                    status = SSLHandshake(context);
                    if status != errSSLWouldBlock {
                        break;
                    }
                }
                if status != 0 {
                    use security_framework_sys::secure_transport::{
                        SSLConnectionRef, SSLGetConnection,
                    };
                    let mut stream_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
                    SSLGetConnection(context, &mut stream_ptr as *mut _ as *mut SSLConnectionRef);
                    if !stream_ptr.is_null() {
                        let _ = Box::from_raw(stream_ptr as *mut TcpStream);
                    }
                    SSLClose(context);
                    CFRelease(context as *const _);
                    return Err(io::Error::other(format!(
                        "SSL handshake failed: {}",
                        status
                    )));
                }
            }

            Ok(TlsStream { context })
        }

        #[cfg(not(target_os = "macos"))]
        compile_error!("Unsupported platform");
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn read_func(
    connection: security_framework_sys::secure_transport::SSLConnectionRef,
    data: *mut std::ffi::c_void,
    data_length: *mut usize,
) -> i32 {
    use security_framework_sys::base::errSecSuccess;
    use security_framework_sys::secure_transport::{errSSLClosedAbort, errSSLWouldBlock};
    unsafe {
        let stream = &mut *(connection as *mut TcpStream);
        let buf = std::slice::from_raw_parts_mut(data as *mut u8, *data_length);

        match stream.read(buf) {
            Ok(n) => {
                *data_length = n;
                errSecSuccess
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                *data_length = 0;
                errSSLWouldBlock
            }
            Err(_) => {
                *data_length = 0;
                // Most useful mappings:
                errSSLClosedAbort
                // Alternatives sometimes seen in wild:
                // -9844  → errSSLConnectionRefused
                // -9806  → errSSLClosedAbort (again, most common)
            }
        }
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn write_func(
    connection: security_framework_sys::secure_transport::SSLConnectionRef,
    data: *const std::ffi::c_void,
    data_length: *mut usize,
) -> i32 {
    use security_framework_sys::base::errSecSuccess;
    use security_framework_sys::secure_transport::errSSLWouldBlock;
    unsafe {
        let stream = &mut *(connection as *mut TcpStream);
        let buf = std::slice::from_raw_parts(data as *const u8, *data_length);
        match stream.write(buf) {
            Ok(n) => {
                *data_length = n;
                errSecSuccess
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                *data_length = 0;
                errSSLWouldBlock
            }
            Err(_) => {
                *data_length = 0;
                -1
            }
        }
    }
}

// MARK: TlsStream
/// A TLS stream
pub struct TlsStream {
    #[cfg(target_os = "macos")]
    context: security_framework_sys::secure_transport::SSLContextRef,
}

impl Read for TlsStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        #[cfg(target_os = "macos")]
        {
            use security_framework_sys::secure_transport::{
                SSLRead, errSSLClosedGraceful, errSSLWouldBlock,
            };
            unsafe {
                let mut processed = 0;
                let status = SSLRead(
                    self.context,
                    buf.as_mut_ptr() as *mut _,
                    buf.len(),
                    &mut processed,
                );
                if status == 0 || status == errSSLClosedGraceful {
                    Ok(processed)
                } else if status == errSSLWouldBlock {
                    if processed > 0 {
                        Ok(processed)
                    } else {
                        Err(io::Error::new(io::ErrorKind::WouldBlock, "Would block"))
                    }
                } else {
                    Err(io::Error::other(format!(
                        "SSL read failed with status: {}",
                        status
                    )))
                }
            }
        }
    }
}

impl Write for TlsStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        #[cfg(target_os = "macos")]
        {
            use security_framework_sys::secure_transport::{SSLWrite, errSSLWouldBlock};
            unsafe {
                let mut processed = 0;
                let status = SSLWrite(
                    self.context,
                    buf.as_ptr() as *const _,
                    buf.len(),
                    &mut processed,
                );
                if status == 0 {
                    Ok(processed)
                } else if status == errSSLWouldBlock {
                    Err(io::Error::new(io::ErrorKind::WouldBlock, "Would block"))
                } else {
                    Err(io::Error::other("SSL write failed"))
                }
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for TlsStream {
    fn drop(&mut self) {
        #[cfg(target_os = "macos")]
        {
            use core_foundation_sys::base::CFRelease;
            use security_framework_sys::secure_transport::{
                SSLClose, SSLConnectionRef, SSLGetConnection,
            };
            unsafe {
                let mut stream_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
                SSLGetConnection(
                    self.context,
                    &mut stream_ptr as *mut _ as *mut SSLConnectionRef,
                );
                if !stream_ptr.is_null() {
                    let _ = Box::from_raw(stream_ptr as *mut TcpStream);
                }
                SSLClose(self.context);
                CFRelease(self.context as *const _);
            }
        }
    }
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    // FIXME: Update test so it doesn't require network access
    #[test]
    fn it_works() {
        use std::io::{Read, Write};
        use std::net::TcpStream;

        let connector = TlsConnector::new().unwrap();

        let stream = TcpStream::connect("example.com:443").unwrap();
        let mut stream = connector.connect("example.com", stream).unwrap();

        stream
            .write_all(b"GET / HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n\r\n")
            .unwrap();
        let mut res = vec![];
        stream.read_to_end(&mut res).unwrap();
        assert!(res.starts_with(b"HTTP/1.1 200 OK"));
    }
}
