# Boykernel

Boykernel is a minimalistic, experimental operating system kernel designed for x86_64 architecture. It is written in Rust and aims to explore low-level programming concepts, including framebuffer manipulation, custom memory management, and kernel-level abstractions.

## Features

- **Framebuffer Rendering**: Displays text and graphics directly on the screen using a framebuffer.
- **No Standard Library**: Operates in a `#![no_std]` environment, suitable for bare-metal development.
- **Custom Target**: Uses a custom `x86_64-unknown-none` target for building the kernel.
- **Panic Handling**: Implements a custom panic handler for kernel-level error handling.

## Getting Started

### Prerequisites

To build and run Boykernel, you need the following tools installed:

- Rust (with `nightly` toolchain)
- `cargo` (Rust package manager)
- `qemu` (for emulation)
- `llvm-tools` (for linking)

### Building the Kernel

1. Clone the repository:
   ```bash
   git clone https://github.com/ofluffydev/boykisser-os
   cd boykisser-os
   ```

2. Add the custom target:
   ```bash
   rustup target add x86_64-unknown-none
   ```

3. Build the kernel:
   ```bash
   make build-kernel
   # or
   make iso
   ```

### Running the Kernel

You can run the kernel using QEMU (Builds the iso automatically):

```bash
make run
```

This will build and launch the kernel in a QEMU virtual machine.

## Code Overview

### Entry Point

The kernel's entry point is the `_start` function in `src/main.rs`. It initializes the framebuffer and renders text on the screen.

### Framebuffer Rendering

The kernel uses a custom `FramebufferInfo` structure to interact with the framebuffer. Characters are rendered using a bitmap font defined in the `FONT` constant.

### Memory Management

A simple `memset` function is implemented for memory initialization.

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests to improve the kernel.

## License

This project is licensed under the parent project License. See the `../LICENSE` file for details.

Other license in projects used within this one:

- **Spleen Font**: The framebuffer console uses the Spleen font. Spleen is licensed under its own terms. You can find the license details in the [Spleen 2.1.0 License](./spleen-2.1.0/LICENSE).
