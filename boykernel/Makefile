RUSTFLAGS = -C relocation-model=static -C link-args=-no-pie

all: release

release:
	RUSTFLAGS="$(RUSTFLAGS)" cargo build -Zbuild-std=core,alloc \
	    --target x86_64-custom.json --release

debug:
	RUSTFLAGS="$(RUSTFLAGS)" cargo build -Zbuild-std=core,alloc \
	    --target x86_64-custom.json

clean:
	cargo clean
