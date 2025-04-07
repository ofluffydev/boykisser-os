#!/bin/bash

# boykisser-os/run.sh

set -e

# Run the build.sh file to build the project
cd boykernel
./build.sh
cd ..

# Run the build.sh in the bootloader directory to run the bootloader and the kernel
BOOTLOADER_SCRIPT="boyloader/build.sh"
if [ -f "$BOOTLOADER_SCRIPT" ]; then
    echo "Running bootloader build script..."
    cd boyloader
    ./build.sh --iso --run
else
    echo "Bootloader build script not found at $BOOTLOADER_SCRIPT"
    exit 1
fi