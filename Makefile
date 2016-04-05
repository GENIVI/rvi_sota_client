MUSL=x86_64-unknown-linux-musl

.PHONY: all
all: pkg/deb/ota-plus-client-0.1.0/bin

pkg/deb/ota-plus-client-0.1.0/bin: target/release/ota_plus_client
	mkdir -p $@
	cp $< $@

target/release/ota_plus_client: src/
	export OPENSSL_STATIC=1
	cargo build --release --target=$(MUSL)
	cp target/$(MUSL)/release/ota_plus_client target/release

.PHONY: deb
deb: pkg/deb/ota-plus-client-0.1.0/bin
	pkg/pkg.sh deb $(CURDIR)

.PHONY: rpm
rpm: target/release/ota_plus_client
	pkg/pkg.sh rpm $(CURDIR)
