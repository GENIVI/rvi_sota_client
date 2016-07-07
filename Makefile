.DEFAULT_GOAL := help
MUSL_TARGET   := x86_64-unknown-linux-musl
GIT_VERSION   := $(shell git rev-parse HEAD | cut -c1-10)

.PHONY: help all run clean test client-release client-musl image deb rpm

help:
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

all: test deb rpm ## Run tests and make new DEB and RPM packages.

run: image ## Run the client inside a Docker container.
	@docker run --rm -it --net=host \
		advancedtelematic/rvi-sota-client:latest

clean: ## Remove all compiled libraries, builds and temporary files.
	@cargo clean
	@rm -f .tmp* *.deb *.rpm pkg/*.deb pkg/*.rpm pkg/*.toml /tmp/ats_credentials.toml

test: ## Run all Cargo tests.
	@cargo test

client-release: src/ ## Make a release build of the client.
	@SERVICE_VERSION=$(GIT_VERSION) cargo build --release

client-musl: src/ ## Make a statically linked release build of the client.
	@docker run --rm \
		--env SERVICE_VERSION=$(GIT_VERSION) \
		--env CARGO_HOME=/cargo \
		--volume ~/.cargo:/cargo \
		--volume $(CURDIR):/build \
		--workdir /build \
		advancedtelematic/rust:latest \
		cargo build --release --target=$(MUSL_TARGET)
	@cp target/$(MUSL_TARGET)/release/ota_plus_client pkg/

image: client-musl ## Build a Docker image from a statically linked binary.
	@docker build -t advancedtelematic/rvi-sota-client pkg

deb: image ## Make a new DEB package inside a Docker container.
	@docker run --rm \
		--env PACKAGE_VERSION=$(GIT_VERSION) \
		--env CARGO_HOME=/cargo \
		--volume ~/.cargo:/cargo \
		--volume $(CURDIR):/build \
		--workdir /build \
		advancedtelematic/rvi-sota-client:latest \
		pkg/pkg.sh deb /build

rpm: image ## Make a new RPM package inside a Docker container.
	@docker run --rm \
		--env PACKAGE_VERSION=$(GIT_VERSION) \
		--env CARGO_HOME=/cargo \
		--volume ~/.cargo:/cargo \
		--volume $(CURDIR):/build \
		--workdir /build \
		advancedtelematic/rvi-sota-client:latest \
		pkg/pkg.sh rpm /build
