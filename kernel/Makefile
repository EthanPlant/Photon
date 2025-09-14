override USER_VARIABLE = $(if $(filter $(origin $(1)), default undefined), $(eval override $(1) := $(2)))

$(call USER_VARIABLE,KARCH,x86_64)

ifeq ($(RUST_TARGET),)
  override RUST_TARGET := $(KARCH)-unknown-none
  ifeq ($(KARCH),riscv64)
	override RUST_TARGET := riscv64gc-unknown-none-elf
  endif
endif

ifeq ($(RUST_PROFILE),)
  override RUST_PROFILE := dev
endif

override RUST_SUBDIR := $(RUST_PROFILE)
ifeq ($(RUST_PROFILE),dev)
  override RUST_SUBDIR = debug
endif

.PHONY: all
all:
	RUSTFLAGS="-C relocation-model=static" cargo build --target $(RUST_TARGET) --profile $(RUST_PROFILE)
	cp $$(find target/$(RUST_TARGET)/$(RUST_SUBDIR) -type f -perm -111 | head -n 1) kernel

.PHONY: clean
clean:
	cargo clean
	rm -rf kernel

.PHONY: check
check:
	RUSTFLAGS="-Dwarnings" cargo check --target $(RUST_TARGET)
	RUSTFLAGS="-Dwarnings" cargo clippy --target $(RUST_TARGET)