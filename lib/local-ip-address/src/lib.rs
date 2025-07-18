/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [local-ip-address](https://crates.io/crates/local-ip-address) crate

use std::net::IpAddr;

/// Returns the local IP address of the machine.
pub fn local_ip() -> Result<IpAddr, std::io::Error> {
    #[cfg(any(
        target_os = "macos",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd"
    ))]
    {
        let socket = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
        if socket < 0 {
            return Err(std::io::Error::last_os_error());
        }

        let mut ifaddrs: *mut libc::ifaddrs = std::ptr::null_mut();
        if unsafe { libc::getifaddrs(&mut ifaddrs) } != 0 {
            unsafe { libc::close(socket) };
            return Err(std::io::Error::last_os_error());
        }
        let mut result = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No valid network interface found",
        ));

        let mut current = ifaddrs;
        while !current.is_null() {
            let addr = unsafe { (*current).ifa_addr };
            let flags = unsafe { (*current).ifa_flags };
            if !addr.is_null()
                && unsafe { (*addr).sa_family } == libc::AF_INET as u8
                && (flags & libc::IFF_LOOPBACK as u32 == 0)
                && (flags & libc::IFF_UP as u32 != 0)
                && (flags & libc::IFF_RUNNING as u32 != 0)
            {
                let mut addr_in = std::mem::MaybeUninit::<libc::sockaddr_in>::uninit();
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        addr as *const u8,
                        addr_in.as_mut_ptr() as *mut u8,
                        size_of::<libc::sockaddr_in>(),
                    );
                    let addr_in = addr_in.assume_init();
                    result = Ok(IpAddr::V4(std::net::Ipv4Addr::from(u32::from_be(
                        addr_in.sin_addr.s_addr,
                    ))));
                    break;
                }
            }
            current = unsafe { (*current).ifa_next };
        }

        unsafe {
            libc::freeifaddrs(ifaddrs);
            libc::close(socket);
        }
        result
    }

    #[cfg(not(any(
        target_os = "macos",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd"
    )))]
    {
        let socket = std::net::UdpSocket::bind((std::net::Ipv4Addr::UNSPECIFIED, 0))?;
        socket
            .connect("1.1.1.1:80")
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let local_addr = socket
            .local_addr()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(local_addr.ip())
    }
}
