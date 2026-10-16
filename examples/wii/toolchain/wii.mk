# Make rules to build a Wii program with clang and lld, include this from an example Makefile:
#
#   NAME := triangles
#   include ../../toolchain/wii.mk
#
# Then `make` builds target/$(NAME).dol from src/*.c and `make run` starts it in Dolphin.

TOOLCHAIN := $(patsubst %/,%,$(dir $(lastword $(MAKEFILE_LIST))))
LIBOGC := $(TOOLCHAIN)/target/libogc
ELF2DOL := $(TOOLCHAIN)/target/elf2dol
GXTEXCONV := $(TOOLCHAIN)/target/gxtexconv

# Apple's clang can't target PowerPC, so prefer Homebrew's LLVM when it's installed
LLVM := $(firstword $(wildcard /opt/homebrew/opt/llvm/bin/ /usr/local/opt/llvm/bin/))
CC := $(LLVM)clang
LD := ld.lld

# The Wii has a PowerPC 750 CPU and there is no OS, so no standard library from the host.
# All code is optimized for size, like the Rust examples (opt-level = "s").
CFLAGS := --target=powerpc-none-eabi -mcpu=750 -ffreestanding -nostdlibinc \
	-ffunction-sections -fdata-sections -DGEKKO -DHW_RVL -Os -g -std=c23 -Wall -Wextra \
	-I$(TOOLCHAIN)/libc/include -I$(LIBOGC)/gc -I$(LIBOGC)/gc/ogc -I$(LIBOGC)/gc/ogc/machine \
	-I$(LIBOGC)/gc/bte -I$(LIBOGC)/gc/wiiuse

OBJECTS := $(patsubst src/%.c,target/%.o,$(wildcard src/*.c))
LIBRARIES := $(addprefix $(TOOLCHAIN)/target/,libogc.a libc.a libcompiler-rt.a)

target/$(NAME).dol: target/$(NAME).elf
	$(ELF2DOL) $< $@

target/$(NAME).elf: $(OBJECTS) $(LIBRARIES)
	$(LD) -T $(TOOLCHAIN)/wii.ld --gc-sections $^ -o $@

target/%.o: src/%.c | $(TOOLCHAIN)/target/libogc.a
	@mkdir -p $(dir $@)
	$(CC) $(CFLAGS) -MMD -MP -c $< -o $@

# The toolchain script knows itself when it needs to rebuild, one run builds all libraries
$(TOOLCHAIN)/target/libogc.a: FORCE
	@$(TOOLCHAIN)/build.sh
$(TOOLCHAIN)/target/libc.a $(TOOLCHAIN)/target/libcompiler-rt.a: $(TOOLCHAIN)/target/libogc.a ;

run: target/$(NAME).elf
	$(TOOLCHAIN)/run.sh $<

clean:
	rm -rf target

.PHONY: run clean FORCE

-include $(OBJECTS:.o=.d)
