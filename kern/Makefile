ROOT := $(shell git rev-parse --show-toplevel)

KERN := kernel
TARGET := target/aarch64-unknown-none/release/${KERN}
TEST_TARGET := aarch64-unknown-none
OBJCPY := cargo objcopy --bin ${KERN} -- --strip-all -O binary
TTY_PATH := /dev/ttyUSB0

.PHONY: all build qemu transmit objdump nm check clean install test

all: bin

bin:
	@mkdir -p build
	cargo objcopy -- --strip-all -O binary build/$(KERN).bin

check:
	@cargo check

elf:
	@cargo build --release
	@mkdir -p build
	@cp -f $(TARGET) build/$(KERN).elf

qemu: bin
	./qemu.sh build/$(KERN).bin

transmit: bin
	@echo "+ Transmitting build/$(KERN).bin to $(TTY_PATH)"
	ttywrite -i build/$(KERN).bin $(TTY_PATH)
	screen $(TTY_PATH) 115200

objdump:
	cargo objdump --release -- --disassemble --no-show-raw-insn

nm: elf
	cargo nm build/$(KERN).elf

clean:
	cargo clean
	rm -rf build

install: build
	@echo "+ Installing build/$(KERN).elf [install-kernel.py]"
	@$(ROOT)/bin/install-kernel.py build/$(KERN).elf

test:
	cargo test -Z build-std --target=$(TEST_TARGET)
