#!/bin/bash

# boykisser-os/boyloader/build.sh

set -e

# Define kernel path
KERNEL_PATH="../boykernel/target/x86_64-custom/release/boykernel"
KERNEL_NAME="boykernel"

# Define OVMF file paths
OVMF_CODE="/usr/share/edk2/x64/OVMF_CODE.4m.fd"
OVMF_VARS="/usr/share/edk2/x64/OVMF_VARS.4m.fd"

# Determine build type
if [[ "$1" == "--flash" ]]; then
    cargo build --release --target x86_64-unknown-uefi
else
    cargo build --target x86_64-unknown-uefi
fi

# Create ESP directory structure
mkdir -p esp/efi/boot

# Copy the bootloader to the ESP
cp target/x86_64-unknown-uefi/debug/boyloader.efi esp/efi/boot/bootx64.efi

# Copy the kernel to the ESP
cp "$KERNEL_PATH" esp/efi/boot/

# Copy UEFI firmware files
cp "$OVMF_CODE" .
cp "$OVMF_VARS" .

# Create FAT image if needed
if [[ "$1" == "--img" || "$1" == "--iso" || "$1" == "--flash" ]]; then
    dd if=/dev/zero of=fat.img bs=1M count=64
    mformat -i fat.img -F ::
    mmd -i fat.img ::/EFI
    mmd -i fat.img ::/EFI/BOOT
    mcopy -i fat.img esp/efi/boot/bootx64.efi ::/EFI/BOOT
    mcopy -i fat.img esp/efi/boot/"$KERNEL_NAME" ::/EFI/BOOT
fi

# Create ISO if needed
if [[ "$1" == "--iso" || "$1" == "--flash" ]]; then
    mkdir -p iso
    cp fat.img iso
    xorriso -as mkisofs -R -f -e fat.img -no-emul-boot -o cdimage.iso iso
fi

# Flash to drive if needed
if [[ "$1" == "--flash" ]]; then
    echo "Available drives:"
    lsblk -d -o NAME,SIZE,MODEL
    echo "WARNING: This will overwrite the selected drive. Proceed with caution!"
    read -p "Enter the drive to flash to (e.g., /dev/sdX): " drive
    if [[ -n "$drive" ]]; then
        sudo dd if=cdimage.iso of="$drive" bs=4M status=progress oflag=sync
        echo "Flashing completed."
    else
        echo "No drive selected. Aborting."
    fi
fi

# Run in QEMU if requested
if [[ "$1" == "--iso" && "$2" == "--run" ]]; then
    qemu-system-x86_64 -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE" \
        -drive format=raw,file=cdimage.iso -smp 4 -m 6G -cpu max \
        -device qemu-xhci -device usb-kbd --serial stdio -M q35 --no-reboot
fi

if [[ "$1" == "--run" && "$1" != "--iso" ]]; then
    qemu-system-x86_64 -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE" \
        -drive format=raw,file=cdimage.iso -smp 4 -m 6G -cpu max \
        -device qemu-xhci -device usb-kbd --serial stdio -M q35 --no-reboot
fi

# ...existing code replaced by the boyloader Makefile...
exit 0
