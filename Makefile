MUSL=x86_64-unknown-linux-musl

.PHONY: all clean ota_plus_client deb rpm

all: deb rpm

clean:
	cargo clean

ota_plus_client: src/
	cargo build --release --target=$(MUSL)
	cp target/$(MUSL)/release/ota_plus_client pkg/

deb: ota_plus_client
	pkg/pkg.sh deb $(CURDIR)

rpm: ota_plus_client
	pkg/pkg.sh rpm $(CURDIR)
