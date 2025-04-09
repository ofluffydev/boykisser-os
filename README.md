# Boykisser OS

Boykisser OS is built to be a decent, daily-drivable, and safe operating system. It demonstrates the basics of kernel and bootloader development using Rust and UEFI.

As said above, the current goals are for the OS to be:

- Decent: Does not create an environmnet so complicated that the user most relearn how to use their computer.
- Daily-drivable: Expected and reproducable behavior for features you'd expect to need on a day-to-day basis.
- Safe operating system: Follows the purity and safety of Rust, creating a stable, memory-safe, and thread-safe kernel and OS.

## Features
- Custom kernel written in Rust.
- UEFI bootloader for modern hardware compatibility.
- Framebuffer-based graphics rendering.
- Simple text rendering on the screen.

## Requirements
- Rust toolchain with nightly version.
- `cargo` for building Rust projects.
- `qemu` for emulation.
- `mtools` and `xorriso` for creating bootable images.
- UEFI firmware files (`OVMF_CODE.fd` and `OVMF_VARS.fd`).

## Building the Project
1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd boykisser-os
   ```

2. Build the kernel:
   ```bash
   make kernel
   ```

3. Build the bootloader:
   ```bash
   make bootloader
   ```

4. Create a bootable ISO:
   ```bash
   make -C boyloader iso
   ```

## Running the OS
To run the OS in QEMU:
```bash
make run
```

## Flashing to a USB Drive
To flash the OS to a USB drive:
```bash
make -C boyloader flash
```
Follow the prompts to select the target drive.

## License
The license for this project is currently undecided. Until a final decision is made (likely the GNU AGPL license),
the project is free to use, modify, and distribute. However, it cannot be made closed-source for any commercial purpose.
