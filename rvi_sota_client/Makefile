.PHONY: release debug docker all clean

SRCS := $(wildcard src/*.rs)
SRCS += Cargo.toml

target/release/sota_client: $(SRCS)
	cargo build --release

target/debug/sota_client: $(SRCS)
	cargo build

docker/sota_client: target/release/sota_client
	cp target/release/sota_client docker

docker: docker/sota_client docker/client.toml
	docker build -t advancedtelematic/sota-client docker

clean:
	rm -f docker/sota_client
	cargo clean

# aliases
debug: target/debug/sota_client
release: target/release/sota_client
all: docker
