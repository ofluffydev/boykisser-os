#!/bin/bash

set -e

# Run the build.sh file to build the project
./build.sh

# Run the build.sh in the bootloader directory to run the bootloader and the kernel
BOOTLOADER_SCRIPT="/home/fluffy/Code/boyloader/build.sh"
if [ -f "$BOOTLOADER_SCRIPT" ]; then
    echo "Running bootloader build script..."
    cd /home/fluffy/Code/boyloader
    ./build.sh --iso --run
else
    echo "Bootloader build script not found at $BOOTLOADER_SCRIPT"
    exit 1
fi