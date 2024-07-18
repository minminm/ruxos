# Building script for stdapps

host_target := $(shell rustc -vV | grep host | cut -d: -f2 | tr -d " ")

sysroot := $(CURDIR)/sysroot
rustlib_dir := $(sysroot)/lib/rustlib/$(TARGET)
rustlib_host_dir := $(sysroot)/lib/rustlib/$(host_target)
rust_src := $(CURDIR)/third_party/rust
log_src := $(CURDIR)/third_party/log
remote_repo_rust := https://github.com/minminm/rust.git
remote_repo_log := https://github.com/minminm/log.git
branch := feature_ruxos

build_std_args := \
  --target $(TARGET) \
  --release \
  --manifest-path $(rust_src)/library/std/Cargo.toml \
  $(verbose)

RUSTFLAGS += \
  --sysroot $(sysroot) \
  -C embed-bitcode=yes \
  -Z force-unstable-if-unmarked \
  --cfg bootstrap

$(rustlib_dir): fetch_code
	@printf "    $(GREEN_C)Creating$(END_C) sysroot\n"
	$(call run_cmd,mkdir,-p $(rustlib_dir) $(rustlib_host_dir))
	$(call run_cmd,ln,-sf $(rust_src)/target/$(TARGET)/release/deps $(rustlib_dir)/lib)
	$(call run_cmd,ln,-sf $(rust_src)/target/release/deps $(rustlib_host_dir)/lib)
	$(call run_cmd,ln,-sf $(CURDIR)/$(TARGET).json $(rustlib_dir)/target.json)

fetch_code:
ifeq ($(wildcard $(rust_src)),)
	@printf "    $(GREEN_C)Cloning$(END_C) rust repository\n"
	git clone $(remote_repo_rust) $(rust_src);
	cd $(rust_src) && git checkout $(branch);
	cd $(rust_src) && git submodule update --init --recursive library/stdarch;
	cd $(rust_src) && git submodule update --init --recursive library/backtrace;
else
	@printf "    $(GREEN_C)Fetching$(END_C) rust repository updates\n"
	cd $(rust_src) && git fetch origin;
	cd $(rust_src) && git checkout $(branch);
	cd $(rust_src) && git pull origin $(branch);
	cd $(rust_src) && git submodule update --init --recursive library/stdarch;
	cd $(rust_src) && git submodule update --init --recursive library/backtrace;
endif
ifeq ($(wildcard $(log_src)),)
	@printf "    $(GREEN_C)Cloning$(END_C) log repository\n"
	git clone $(remote_repo_log) $(log_src);
	cd $(log_src) && git checkout $(branch);
else
	@printf "    $(GREEN_C)Fetching$(END_C) log repository updates\n"
	cd $(rust_src) && git fetch origin;
	cd $(rust_src) && git checkout $(branch);
	cd $(rust_src) && git pull origin $(branch);
endif

build_std: $(rustlib_dir)
	@printf "    $(GREEN_C)Building$(END_C) rust-std ($(TARGET))\n"
#	stage 1: build the core and alloc libraries first which are required by RuxOS.
	$(call run_cmd, cargo build,$(build_std_args) -p core -p alloc --features "compiler-builtins-mem" --verbose)
#	stage 2: build RuxOS and the std library with specified features
#	$(call run_cmd,cargo build,$(build_std_args) -p std --features "compiler-builtins-mem $(RUX_FEAT) $(LIB_FEAT) ruxfeat/sched_rr arceos_api/irq")
	$(call run_cmd, cargo build,$(build_std_args) -p std --features "compiler-builtins-mem $(RUX_FEAT) $(LIB_FEAT) ruxfeat/sched_rr ruxfeat/tls rust_std_api/irq" --verbose)

# RUSTFLAGS: "-C link-arg=-T/mnt/c/Users/MIN/Documents/ruxos/modules/ruxhal/linker_aarch64-qemu-virt.lds -C link-arg=-no-pie --sysroot /mnt/c/Users/MIN/Documents/ruxos/sysroot -C embed-bitcode=yes -Z force-unstable-if-unmarked"