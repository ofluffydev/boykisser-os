all: debug

debug:
	cargo build --target x86_64-unknown-uefi

release:
	cargo build --release --target x86_64-unknown-uefi

# Replicate ESP copying logic here, but let the main Makefile handle ISO creation
esp:
	mkdir -p esp/efi/boot
	cp target/x86_64-unknown-uefi/debug/boyloader.efi esp/efi/boot/bootx64.efi
	cp ../boykernel/target/x86_64-custom/release/boykernel esp/efi/boot/

clean:
	cargo clean
	rm -rf esp
