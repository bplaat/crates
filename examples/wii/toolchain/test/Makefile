NAME := test
include ../wii.mk

# Convert the test image to a TPL texture file with gxtexconv
target/test.tpl: src/assets/test.scf src/assets/test.png | $(TOOLCHAIN)/target/libogc.a
	@mkdir -p target
	cd target && $(abspath $(GXTEXCONV)) -s ../src/assets/test.scf -d test.d -o test.tpl

target/main.o: target/test.tpl
