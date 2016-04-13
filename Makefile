MUSL=x86_64-unknown-linux-musl

.PHONY: all
all: ota_plus_client

.PHONY: ota_plus_client
ota_plus_client: src/
	cargo build --release --target=$(MUSL)
	mkdir -p pkg/deb/ota-plus-client-0.1.0/bin
	cp target/$(MUSL)/release/ota_plus_client pkg/deb/ota-plus-client-0.1.0/bin/
	cp target/$(MUSL)/release/ota_plus_client pkg/rpm/

.PHONY: deb
deb: ota_plus_client
	pkg/pkg.sh deb $(CURDIR)

.PHONY: rpm
rpm: ota_plus_client
	pkg/pkg.sh rpm $(CURDIR)
