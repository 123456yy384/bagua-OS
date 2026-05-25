# 八卦 OS 构建脚本
# 需要: Rust nightly, x86_64-unknown-none target, QEMU (可选)

KERNEL_DIR = kernel
TARGET = x86_64-unknown-none
KERNEL_BIN = $(KERNEL_DIR)/target/$(TARGET)/release/bagua_os_kernel

.PHONY: all kernel run test clean

all: kernel test

# 构建裸机内核
kernel:
	cd $(KERNEL_DIR) && cargo +nightly build --target $(TARGET) --release
	@echo "[OK] Kernel built: $(KERNEL_BIN)"

# 运行 QEMU（需要 QEMU 7.x 或更早 — 8.x+ 不支持 Multiboot2 -kernel）
# 替代: make sim（用户态模拟器，调度器一致）或用 Limine 制作 ISO
run: kernel
	@echo "[INFO] QEMU 8.x+ 不支持 Multiboot2 -kernel。使用 make sim 或 Limine ISO。"
	@echo "[TRY] qemu-system-x86_64 -kernel $(KERNEL_BIN) -serial stdio -no-reboot -no-shutdown -m 256M"

# 带 GDB 调试
debug: kernel
	qemu-system-x86_64 \
		-kernel $(KERNEL_BIN) \
		-serial stdio \
		-s -S \
		-no-reboot \
		-m 256M

# 运行用户态模拟器
sim:
	cargo run --bin bagua-os

# 运行测试
test:
	cargo test

# 清理
clean:
	cd $(KERNEL_DIR) && cargo clean
	cargo clean

# 检查
check:
	cargo check
	cd $(KERNEL_DIR) && cargo +nightly check --target $(TARGET) --release
