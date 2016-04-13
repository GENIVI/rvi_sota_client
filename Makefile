MUSL=x86_64-unknown-linux-musl

.PHONY: all
all: ota_plus_client

.PHONY: ota_plus_client
ota_plus_client: src/
	cargo build --release --target=$(MUSL)
	cp target/$(MUSL)/release/ota_plus_client pkg/

.PHONY: deb
deb: ota_plus_client
	pkg/pkg.sh deb $(CURDIR)

.PHONY: rpm
rpm: ota_plus_client
	pkg/pkg.sh rpm $(CURDIR)
