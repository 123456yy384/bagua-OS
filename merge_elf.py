import struct, subprocess, sys

RUST = "/mnt/c/Users/LENOVO/bagua-os/kernel/target/x86_64-unknown-none/release/bagua-os-kernel"
MIN = "/tmp/mb2_test.elf"
OUT = "/mnt/c/Users/LENOVO/bagua-os/kernel/target/x86_64-unknown-none/release/bagua-os-kernel-bootable"

# Get Rust flat binary
subprocess.run(["objcopy", "-O", "binary", RUST, "/tmp/rust-full.bin"], check=True)
with open("/tmp/rust-full.bin", "rb") as f:
    rust_body = bytearray(f.read())
print(f"Rust body: {len(rust_body)} bytes")

# Read minimal kernel ELF
with open(MIN, "rb") as f:
    min_elf = bytearray(f.read())

# Build merged ELF: ELF header + PHDRs (up to file offset 0x1000), then Rust body
e_phoff = struct.unpack_from("<Q", min_elf, 32)[0]

result = bytearray(min_elf[:0x1000])  # ELF header, PHDRs, padding
result.extend(rust_body)              # Rust body at file offset 0x1000

# Set entry to start_32 (VA 0x100030)
struct.pack_into("<Q", result, 24, 0x100030)

# Update LOAD segment
total_filesz = len(rust_body)
total_memsz = total_filesz + 0x9000  # +BSS estimate
struct.pack_into("<Q", result, e_phoff + 32, total_filesz)
struct.pack_into("<Q", result, e_phoff + 40, total_memsz)

with open(OUT, "wb") as f:
    f.write(result)
print(f"Written: {OUT} ({len(result)} bytes)")
