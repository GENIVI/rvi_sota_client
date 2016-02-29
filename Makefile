.PHONY: all

all: pkg/deb/ota-plus-client-0.1.0/bin

pkg/deb/ota-plus-client-0.1.0/bin: target/release/ota-plus-client
	mkdir -p $@
	cp $< $@

target/release/ota-plus-client: src/
	cargo build --release

