// Everything the C library uses from libogc, the same hooks devkitPPC's newlib uses. The
// other way around libogc calls build_argv and __crtmain from crt.c and the C functions.

#pragma once

#include <gctypes.h>
#include <ogc/irq.h>
#include <ogc/system.h>
#include <time.h>

// Leaves the program, back to the loader or the system menu
void __syscall_exit(int status);
// Puts the current thread to sleep
int __syscall_nanosleep(const struct timespec* request, struct timespec* remaining);
// Initializes IOS and the system settings, the main thread calls this before main()
void SYS_PreMain(void);
