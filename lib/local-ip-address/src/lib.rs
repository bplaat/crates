/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [local-ip-address](https://crates.io/crates/local-ip-address) crate

#![allow(non_camel_case_types)]
#![allow(clippy::upper_case_acronyms)]

use std::net::IpAddr;

/// Returns the local IPv4 address of the machine.
pub fn local_ip() -> Result<IpAddr, std::io::Error> {
    // MARK: Unix
    #[cfg(unix)]
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
                && unsafe { (*addr).sa_family } as u32 == libc::AF_INET as u32
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
                    )
                };
                let addr_in = unsafe { addr_in.assume_init() };
                result = Ok(IpAddr::V4(std::net::Ipv4Addr::from(u32::from_be(
                    addr_in.sin_addr.s_addr,
                ))));
                break;
            }
            current = unsafe { (*current).ifa_next };
        }

        unsafe { libc::freeifaddrs(ifaddrs) };
        unsafe { libc::close(socket) };
        result
    }

    // MARK: Windows
    #[cfg(windows)]
    {
        use std::ffi::c_void;

        const AF_UNSPEC: u32 = 0;
        const AF_INET: u32 = 2;
        const GAA_FLAG_SKIP_ANYCAST: u32 = 0x0002;
        const GAA_FLAG_SKIP_MULTICAST: u32 = 0x0004;
        const GAA_FLAG_SKIP_DNS_SERVER: u32 = 0x0008;
        const ERROR_BUFFER_OVERFLOW: u32 = 111;
        const NO_ERROR: u32 = 0;

        #[repr(C)]
        struct SOCKET_ADDRESS {
            lp_socket_addr: *mut SOCKADDR,
            i_socket_addr_length: i32,
        }

        #[repr(C)]
        struct SOCKADDR {
            sa_family: u16,
            sa_data: [u8; 14],
        }

        #[repr(C)]
        struct SOCKADDR_IN {
            sin_family: u16,
            sin_port: u16,
            sin_addr: IN_ADDR,
            sin_zero: [u8; 8],
        }

        #[repr(C)]
        struct IN_ADDR {
            s_addr: u32,
        }

        #[repr(C)]
        struct IP_ADAPTER_UNICAST_ADDRESS {
            length: u32,
            flags: u32,
            next: *mut IP_ADAPTER_UNICAST_ADDRESS,
            address: SOCKET_ADDRESS,
            prefix_origin: i32,
            suffix_origin: i32,
            dad_state: i32,
            valid_lifetime: u32,
            preferred_lifetime: u32,
            lease_lifetime: u32,
            on_link_prefix_length: u8,
        }

        #[repr(C)]
        struct IP_ADAPTER_ADDRESSES {
            length: u32,
            if_index: u32,
            next: *mut IP_ADAPTER_ADDRESSES,
            adapter_name: *mut i8,
            first_unicast_address: *mut IP_ADAPTER_UNICAST_ADDRESS,
            // ... more fields exist but we don't need them
        }

        #[link(name = "iphlpapi")]
        unsafe extern "system" {
            fn GetAdaptersAddresses(
                family: u32,
                flags: u32,
                reserved: *mut c_void,
                adapter_addresses: *mut IP_ADAPTER_ADDRESSES,
                size_pointer: *mut u32,
            ) -> u32;
        }

        // First call to get the required buffer size
        let mut buffer_size: u32 = 0;
        let result = unsafe {
            GetAdaptersAddresses(
                AF_UNSPEC,
                GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST | GAA_FLAG_SKIP_DNS_SERVER,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut buffer_size,
            )
        };
        if result != ERROR_BUFFER_OVERFLOW && result != NO_ERROR {
            return Err(std::io::Error::from_raw_os_error(result as i32));
        }

        // Second call to get the actual data
        let mut buffer: Vec<u8> = vec![0u8; buffer_size as usize];
        let adapter_addresses = buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES;
        let result = unsafe {
            GetAdaptersAddresses(
                AF_UNSPEC,
                GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST | GAA_FLAG_SKIP_DNS_SERVER,
                std::ptr::null_mut(),
                adapter_addresses,
                &mut buffer_size,
            )
        };
        if result != NO_ERROR {
            return Err(std::io::Error::from_raw_os_error(result as i32));
        }

        // Iterate through adapters
        let mut current_adapter = adapter_addresses;
        let mut best_ipv4_addr: Option<IpAddr> = None;
        while !current_adapter.is_null() {
            let adapter = unsafe { &*current_adapter };

            // Iterate through unicast addresses
            let mut current_address = adapter.first_unicast_address;
            while !current_address.is_null() {
                let unicast_addr = unsafe { &*current_address };
                let socket_addr = unicast_addr.address.lp_socket_addr;
                if !socket_addr.is_null() {
                    let sockaddr_in = socket_addr as *const SOCKADDR_IN;
                    let ip_bytes = unsafe { (*sockaddr_in).sin_addr.s_addr.to_ne_bytes() };
                    let family = unsafe { (*socket_addr).sa_family } as u32;
                    if family == AF_INET
                        && ip_bytes[0] != 127
                        && !(ip_bytes[0] == 169 && ip_bytes[1] == 254)
                    {
                        best_ipv4_addr = Some(IpAddr::from(ip_bytes));
                    }
                }
                current_address = unsafe { (*current_address).next };
            }
            current_adapter = adapter.next;
        }

        if let Some(ipv4) = best_ipv4_addr {
            return Ok(ipv4);
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No local IP address found",
        ))
    }

    // MARK: Others
    #[cfg(not(any(unix, windows)))]
    {
        compile_error!("Unsupported platform");
    }
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_local_ip() {
        let ip = local_ip().expect("Failed to get local IP address");
        assert!(matches!(ip, IpAddr::V4(_)));
    }
}
