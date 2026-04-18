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
#[allow(unsafe_code)]
pub fn local_ip() -> Result<IpAddr, std::io::Error> {
    cfg_select! {
        unix => {
            let mut ifaddrs: *mut libc::ifaddrs = std::ptr::null_mut();
            // SAFETY: ifaddrs is a valid out-pointer; getifaddrs will initialize it on success.
            if unsafe { libc::getifaddrs(&mut ifaddrs) } != 0 {
                return Err(std::io::Error::last_os_error());
            }
            let mut result = Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No valid network interface found",
            ));

            let mut current = ifaddrs;
            while !current.is_null() {
                // SAFETY: current is non-null (checked in the while condition), pointing to a valid ifaddrs node.
                let addr = unsafe { (*current).ifa_addr };
                // SAFETY: current is non-null, as above.
                let flags = unsafe { (*current).ifa_flags };
                if !addr.is_null()
                    // SAFETY: addr is non-null (checked above), pointing to a valid sockaddr.
                    && unsafe { (*addr).sa_family } as u32 == libc::AF_INET as u32
                    && (flags & libc::IFF_LOOPBACK as u32 == 0)
                    && (flags & libc::IFF_UP as u32 != 0)
                    && (flags & libc::IFF_RUNNING as u32 != 0)
                {
                    let mut addr_in = std::mem::MaybeUninit::<libc::sockaddr_in>::uninit();
                    // SAFETY: addr points to a valid sockaddr_in (verified via AF_INET family check); addr_in is a correctly sized MaybeUninit target.
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            addr as *const u8,
                            addr_in.as_mut_ptr() as *mut u8,
                            size_of::<libc::sockaddr_in>(),
                        )
                    };
                    // SAFETY: addr_in was fully initialized by copy_nonoverlapping above.
                    let addr_in = unsafe { addr_in.assume_init() };
                    result = Ok(IpAddr::V4(std::net::Ipv4Addr::from(u32::from_be(
                        addr_in.sin_addr.s_addr,
                    ))));
                    break;
                }
                // SAFETY: current is non-null; ifa_next is a valid pointer to the next node or null.
                current = unsafe { (*current).ifa_next };
            }

            // SAFETY: ifaddrs was obtained from a successful getifaddrs call and has not been freed yet.
            unsafe { libc::freeifaddrs(ifaddrs) };
            result
        }
        windows => {
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
            // SAFETY: null adapter_addresses pointer is valid for the size-query call; all other arguments are correct constants.
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
            // SAFETY: buffer is sized to buffer_size bytes as reported by the previous call; adapter_addresses points into it.
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
                // SAFETY: current_adapter is non-null (checked in while condition), pointing to a valid IP_ADAPTER_ADDRESSES node.
                let adapter = unsafe { &*current_adapter };

                // Iterate through unicast addresses
                let mut current_address = adapter.first_unicast_address;
                while !current_address.is_null() {
                    // SAFETY: current_address is non-null (checked in while condition), pointing to a valid IP_ADAPTER_UNICAST_ADDRESS node.
                    let unicast_addr = unsafe { &*current_address };
                    let socket_addr = unicast_addr.address.lp_socket_addr;
                    if !socket_addr.is_null() {
                        // SAFETY: socket_addr is non-null (checked above); sa_family is the first field of SOCKADDR.
                        let family = unsafe { (*socket_addr).sa_family } as u32;
                        if family == AF_INET {
                            let sockaddr_in = socket_addr as *const SOCKADDR_IN;
                            // SAFETY: family confirmed this points to a valid SOCKADDR_IN, so reading sin_addr is sound.
                            let ip_bytes = unsafe { (*sockaddr_in).sin_addr.s_addr.to_ne_bytes() };
                            if ip_bytes[0] != 127 && !(ip_bytes[0] == 169 && ip_bytes[1] == 254) {
                                best_ipv4_addr = Some(IpAddr::from(ip_bytes));
                            }
                        }
                    }
                    // SAFETY: current_address is non-null; next is a valid pointer to the next node or null.
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
        _ => {
            compile_error!("Unsupported platform")
        }
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
