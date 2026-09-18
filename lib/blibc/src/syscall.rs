use core::ffi::c_long;

use crate::errno::convert_syscall_result;

cfg_select! {
    target_arch = "x86_64" => {
        use core::arch::asm;

        pub(crate) unsafe fn syscall1(number: usize, arg1: usize) -> isize {
            let result: isize;
            unsafe {
                asm!(
                    "syscall",
                    inlateout("rax") number as isize => result,
                    in("rdi") arg1,
                    lateout("rcx") _,
                    lateout("r11") _,
                    options(nostack),
                );
            }
            result
        }

        pub(crate) unsafe fn syscall3(
            number: usize,
            arg1: usize,
            arg2: usize,
            arg3: usize,
        ) -> isize {
            let result: isize;
            unsafe {
                asm!(
                    "syscall",
                    inlateout("rax") number as isize => result,
                    in("rdi") arg1,
                    in("rsi") arg2,
                    in("rdx") arg3,
                    lateout("rcx") _,
                    lateout("r11") _,
                    options(nostack),
                );
            }
            result
        }

        pub(crate) unsafe fn syscall6(
            number: usize,
            arg1: usize,
            arg2: usize,
            arg3: usize,
            arg4: usize,
            arg5: usize,
            arg6: usize,
        ) -> isize {
            let result: isize;
            unsafe {
                asm!(
                    "syscall",
                    inlateout("rax") number as isize => result,
                    in("rdi") arg1,
                    in("rsi") arg2,
                    in("rdx") arg3,
                    in("r10") arg4,
                    in("r8") arg5,
                    in("r9") arg6,
                    lateout("rcx") _,
                    lateout("r11") _,
                    options(nostack),
                );
            }
            result
        }
    }
    target_arch = "aarch64" => {
        use core::arch::asm;

        pub(crate) unsafe fn syscall1(number: usize, arg1: usize) -> isize {
            let result: isize;
            unsafe {
                asm!(
                    "svc #0",
                    in("x8") number,
                    inlateout("x0") arg1 as isize => result,
                    options(nostack),
                );
            }
            result
        }

        pub(crate) unsafe fn syscall3(
            number: usize,
            arg1: usize,
            arg2: usize,
            arg3: usize,
        ) -> isize {
            let result: isize;
            unsafe {
                asm!(
                    "svc #0",
                    in("x8") number,
                    inlateout("x0") arg1 as isize => result,
                    in("x1") arg2,
                    in("x2") arg3,
                    options(nostack),
                );
            }
            result
        }

        pub(crate) unsafe fn syscall6(
            number: usize,
            arg1: usize,
            arg2: usize,
            arg3: usize,
            arg4: usize,
            arg5: usize,
            arg6: usize,
        ) -> isize {
            let result: isize;
            unsafe {
                asm!(
                    "svc #0",
                    in("x8") number,
                    inlateout("x0") arg1 as isize => result,
                    in("x1") arg2,
                    in("x2") arg3,
                    in("x3") arg4,
                    in("x4") arg5,
                    in("x5") arg6,
                    options(nostack),
                );
            }
            result
        }
    }
    _ => {
        compile_error!("unsupported target architecture");
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn syscall(
    number: c_long,
    arg1: c_long,
    arg2: c_long,
    arg3: c_long,
    arg4: c_long,
    arg5: c_long,
    arg6: c_long,
) -> c_long {
    convert_syscall_result(unsafe {
        syscall6(
            number as usize,
            arg1 as usize,
            arg2 as usize,
            arg3 as usize,
            arg4 as usize,
            arg5 as usize,
            arg6 as usize,
        )
    }) as c_long
}
