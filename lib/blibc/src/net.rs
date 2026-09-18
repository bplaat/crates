use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use crate::errno::convert_syscall_result;
use crate::io::{close, ioctl};
use crate::syscall::{syscall3, syscall6};

cfg_select! {
    target_arch = "x86_64" => {
        const SYS_SOCKET: usize = 41;
        const SYS_CONNECT: usize = 42;
        const SYS_SENDTO: usize = 44;
        const SYS_RECVFROM: usize = 45;
        const SYS_BIND: usize = 49;
        const SYS_LISTEN: usize = 50;
        const SYS_GETSOCKNAME: usize = 51;
        const SYS_GETPEERNAME: usize = 52;
        const SYS_SOCKETPAIR: usize = 53;
        const SYS_SETSOCKOPT: usize = 54;
        const SYS_GETSOCKOPT: usize = 55;
        const SYS_SENDMSG: usize = 46;
        const SYS_RECVMSG: usize = 47;
        const SYS_SHUTDOWN: usize = 48;
        const SYS_ACCEPT4: usize = 288;
    }
    target_arch = "aarch64" => {
        const SYS_SOCKET: usize = 198;
        const SYS_SOCKETPAIR: usize = 199;
        const SYS_BIND: usize = 200;
        const SYS_LISTEN: usize = 201;
        const SYS_CONNECT: usize = 203;
        const SYS_GETSOCKNAME: usize = 204;
        const SYS_GETPEERNAME: usize = 205;
        const SYS_SENDTO: usize = 206;
        const SYS_RECVFROM: usize = 207;
        const SYS_SETSOCKOPT: usize = 208;
        const SYS_GETSOCKOPT: usize = 209;
        const SYS_SHUTDOWN: usize = 210;
        const SYS_SENDMSG: usize = 211;
        const SYS_RECVMSG: usize = 212;
        const SYS_ACCEPT4: usize = 242;
    }
    _ => {
        compile_error!("unsupported target architecture");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn socket(domain: c_int, kind: c_int, protocol: c_int) -> c_int {
    convert_syscall_result(unsafe {
        syscall3(
            SYS_SOCKET,
            domain as usize,
            kind as usize,
            protocol as usize,
        )
    }) as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn socketpair(
    domain: c_int,
    kind: c_int,
    protocol: c_int,
    sockets: *mut c_int,
) -> c_int {
    convert_syscall_result(unsafe {
        syscall6(
            SYS_SOCKETPAIR,
            domain as usize,
            kind as usize,
            protocol as usize,
            sockets as usize,
            0,
            0,
        )
    }) as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sendmsg(fd: c_int, message: *const c_void, flags: c_int) -> isize {
    convert_syscall_result(unsafe {
        syscall3(SYS_SENDMSG, fd as usize, message as usize, flags as usize)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn recvmsg(fd: c_int, message: *mut c_void, flags: c_int) -> isize {
    convert_syscall_result(unsafe {
        syscall3(SYS_RECVMSG, fd as usize, message as usize, flags as usize)
    })
}

macro_rules! socket_address_call {
    ($name:ident, $number:ident) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(fd: c_int, address: *const c_void, length: u32) -> c_int {
            convert_syscall_result(unsafe {
                syscall3($number, fd as usize, address as usize, length as usize)
            }) as c_int
        }
    };
}

socket_address_call!(bind, SYS_BIND);
socket_address_call!(connect, SYS_CONNECT);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn listen(fd: c_int, backlog: c_int) -> c_int {
    convert_syscall_result(unsafe { syscall3(SYS_LISTEN, fd as usize, backlog as usize, 0) })
        as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn shutdown(fd: c_int, how: c_int) -> c_int {
    convert_syscall_result(unsafe { syscall3(SYS_SHUTDOWN, fd as usize, how as usize, 0) }) as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn accept4(
    fd: c_int,
    address: *mut c_void,
    length: *mut u32,
    flags: c_int,
) -> c_int {
    convert_syscall_result(unsafe {
        syscall6(
            SYS_ACCEPT4,
            fd as usize,
            address as usize,
            length as usize,
            flags as usize,
            0,
            0,
        )
    }) as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn send(
    fd: c_int,
    buffer: *const c_void,
    length: usize,
    flags: c_int,
) -> isize {
    convert_syscall_result(unsafe {
        syscall6(
            SYS_SENDTO,
            fd as usize,
            buffer as usize,
            length,
            flags as usize,
            0,
            0,
        )
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn recv(
    fd: c_int,
    buffer: *mut c_void,
    length: usize,
    flags: c_int,
) -> isize {
    convert_syscall_result(unsafe {
        syscall6(
            SYS_RECVFROM,
            fd as usize,
            buffer as usize,
            length,
            flags as usize,
            0,
            0,
        )
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn getpeername(fd: c_int, address: *mut c_void, length: *mut u32) -> c_int {
    convert_syscall_result(unsafe {
        syscall3(
            SYS_GETPEERNAME,
            fd as usize,
            address as usize,
            length as usize,
        )
    }) as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn getsockname(fd: c_int, address: *mut c_void, length: *mut u32) -> c_int {
    convert_syscall_result(unsafe {
        syscall3(
            SYS_GETSOCKNAME,
            fd as usize,
            address as usize,
            length as usize,
        )
    }) as c_int
}

macro_rules! socket_option_call {
    ($name:ident, $number:ident, $pointer:ty) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(
            fd: c_int,
            level: c_int,
            option: c_int,
            value: $pointer,
            length: *mut u32,
        ) -> c_int {
            convert_syscall_result(unsafe {
                syscall6(
                    $number,
                    fd as usize,
                    level as usize,
                    option as usize,
                    value as usize,
                    length as usize,
                    0,
                )
            }) as c_int
        }
    };
}

socket_option_call!(getsockopt, SYS_GETSOCKOPT, *mut c_void);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn setsockopt(
    fd: c_int,
    level: c_int,
    option: c_int,
    value: *const c_void,
    length: u32,
) -> c_int {
    convert_syscall_result(unsafe {
        syscall6(
            SYS_SETSOCKOPT,
            fd as usize,
            level as usize,
            option as usize,
            value as usize,
            length as usize,
            0,
        )
    }) as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn getaddrinfo(
    _node: *const c_char,
    _service: *const c_char,
    _hints: *const c_void,
    result: *mut *mut c_void,
) -> c_int {
    if !result.is_null() {
        unsafe { result.write(ptr::null_mut()) };
    }
    -2
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn freeaddrinfo(_result: *mut c_void) {}

#[unsafe(no_mangle)]
pub extern "C" fn gai_strerror(_error: c_int) -> *const c_char {
    c"name resolution is unavailable".as_ptr()
}

#[unsafe(no_mangle)]
pub extern "C" fn __res_init() -> c_int {
    0
}

#[repr(C)]
struct ifaddrs {
    ifa_next: *mut ifaddrs,
    ifa_name: *mut c_char,
    ifa_flags: u32,
    ifa_addr: *mut c_void,
    ifa_netmask: *mut c_void,
    ifa_broadaddr: *mut c_void,
    ifa_data: *mut c_void,
}

#[repr(C)]
struct ifreq {
    ifr_name: [c_char; 16],
    ifr_data: [u8; 24],
}

static mut INTERFACE_NAME: [c_char; 5] = [
    b'e' as c_char,
    b't' as c_char,
    b'h' as c_char,
    b'0' as c_char,
    0,
];
static mut INTERFACE_ADDRESS: [u8; 16] = [0; 16];
static mut INTERFACE: ifaddrs = ifaddrs {
    ifa_next: ptr::null_mut(),
    ifa_name: ptr::null_mut(),
    ifa_flags: 0,
    ifa_addr: ptr::null_mut(),
    ifa_netmask: ptr::null_mut(),
    ifa_broadaddr: ptr::null_mut(),
    ifa_data: ptr::null_mut(),
};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn getifaddrs(output: *mut *mut c_void) -> c_int {
    const AF_INET: c_int = 2;
    const SOCK_DGRAM: c_int = 2;
    const SIOCGIFADDR: usize = 0x8915;
    const IFF_UP: u32 = 0x1;
    const IFF_RUNNING: u32 = 0x40;

    let fd = unsafe { socket(AF_INET, SOCK_DGRAM, 0) };
    if fd < 0 {
        return -1;
    }
    let mut request = ifreq {
        ifr_name: [0; 16],
        ifr_data: [0; 24],
    };
    request.ifr_name[..4].copy_from_slice(&[
        b'e' as c_char,
        b't' as c_char,
        b'h' as c_char,
        b'0' as c_char,
    ]);
    let result = unsafe { ioctl(fd, SIOCGIFADDR, (&raw mut request).cast()) };
    unsafe { close(fd) };
    if result != 0 {
        return -1;
    }
    unsafe {
        ptr::copy_nonoverlapping(
            request.ifr_data.as_ptr(),
            ptr::addr_of_mut!(INTERFACE_ADDRESS).cast::<u8>(),
            16,
        );
        INTERFACE.ifa_name = ptr::addr_of_mut!(INTERFACE_NAME).cast::<c_char>();
        INTERFACE.ifa_flags = IFF_UP | IFF_RUNNING;
        INTERFACE.ifa_addr = ptr::addr_of_mut!(INTERFACE_ADDRESS).cast();
        output.write(ptr::addr_of_mut!(INTERFACE).cast());
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn freeifaddrs(_interfaces: *mut c_void) {}
