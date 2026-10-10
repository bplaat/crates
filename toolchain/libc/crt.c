// C runtime startup, libogc's crt0 sets up the system and starts the main thread at __crtmain
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#include "libogc.h"

int main(int argc, char** argv);

// Splits the null separated command line from the loader into argv, libogc calls this at startup
void build_argv(struct __argv* args) {
    char** argv = (char**)(((uintptr_t)args->commandLine + args->length + 3) & ~(uintptr_t)3);
    char* arg = args->commandLine;
    int argc = 0;
    for (int length = args->length; length > 0;) {
        argv[argc++] = arg;
        int size = (int)strlen(arg) + 1;
        arg += size;
        length -= size;
    }
    args->argc = argc;
    args->argv = argv;
    args->endARGV = &argv[argc];
}

void __crtmain(void) {
    SYS_PreMain();
    exit(main(__system_argv->argc, __system_argv->argv));
}
