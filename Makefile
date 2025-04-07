# Variables
OVMF_CODE = /usr/share/edk2/x64/OVMF_CODE.4m.fd
OVMF_VARS = /usr/share/edk2/x64/OVMF_VARS.4m.fd
KERNEL_PATH = boykernel/target/x86_64-custom/release/boykernel
KERNEL_NAME = boykernel
FAT_IMG = fat.img
ISO_FILE = cdimage.iso

all: kernel bootloader

kernel:
	@$(MAKE) -C boykernel

bootloader:
	@$(MAKE) -C boyloader

iso: all
	@rm -f $(FAT_IMG) $(ISO_FILE)
	dd if=/dev/zero of=$(FAT_IMG) bs=1M count=64
	mformat -i $(FAT_IMG) -F ::
	mmd -i $(FAT_IMG) ::/EFI
	mmd -i $(FAT_IMG) ::/EFI/BOOT
	mkdir -p esp/efi/boot
	cp boyloader/target/x86_64-unknown-uefi/debug/boyloader.efi esp/efi/boot/bootx64.efi
	cp $(KERNEL_PATH) esp/efi/boot/$(KERNEL_NAME)
	mcopy -i $(FAT_IMG) esp/efi/boot/bootx64.efi ::/EFI/BOOT
	mcopy -i $(FAT_IMG) esp/efi/boot/$(KERNEL_NAME) ::/EFI/BOOT
	mkdir -p iso
	cp $(FAT_IMG) iso
	xorriso -as mkisofs -R -f -e $(FAT_IMG) -no-emul-boot -o $(ISO_FILE) iso

run: iso
	@cd boyloader && qemu-system-x86_64 -drive if=pflash,format=raw,readonly=on,file="$(OVMF_CODE)" \
	    -drive format=raw,file=cdimage.iso -smp 4 -m 6G -cpu max \
	    -device qemu-xhci -device usb-kbd --serial stdio -M q35 --no-reboot

flash: iso
	@echo "WARNING: This overwrites the selected drive!"
	@lsblk -d -o NAME,SIZE,MODEL
	@read -p "Drive to flash to (e.g. /dev/sdX): " drive; \
	 if [ -n "$$drive" ]; then \
	   sudo dd if=$(ISO_FILE) of="$$drive" bs=4M status=progress oflag=sync; \
	 fi

clean:
	rm -rf iso $(ISO_FILE) esp $(FAT_IMG) bootx64.efi $(KERNEL_NAME)
	@$(MAKE) clean -C boykernel
	@$(MAKE) clean -C boyloader
