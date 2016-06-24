.DEFAULT_GOAL := help
GIT_VERSION   := $(shell git rev-parse HEAD | cut -c1-10)
MUSL_TARGET   := x86_64-unknown-linux-musl

.PHONY: help all run clean version test client-release client-musl image deb rpm

help:
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

all: clean test deb rpm ## Clean, test and make new DEB and RPM packages.

run: image ## Run the client inside a Docker container.
	@docker run --rm -it --net=host \
		advancedtelematic/ota-plus-client:latest

clean: ## Remove all compiled libraries, builds and temporary files.
	@cargo clean
	@rm -f .tmp* src/.version

version:
	@printf $(GIT_VERSION) > src/.version

test: version ## Run all Cargo tests.
	@cargo test

client-release: src/ version ## Make a release build of the client.
	@cargo build --release

client-musl: src/ version ## Make a statically linked release build of the client.
	@docker run --rm \
		--env CARGO_HOME=/cargo \
		--volume ~/.cargo:/cargo \
		--volume $(CURDIR):/build \
		--workdir /build \
		advancedtelematic/rust:latest \
		cargo build --release --target=$(MUSL_TARGET)
	@cp target/$(MUSL_TARGET)/release/ota_plus_client pkg/

image: client-musl ## Build a Docker image from a statically linked binary.
	@docker build -t advancedtelematic/ota-plus-client pkg

deb: image ## Make a new DEB package inside a Docker container.
	@docker run --rm \
		--env CARGO_HOME=/cargo \
		--volume ~/.cargo:/cargo \
		--volume $(CURDIR):/build \
		--workdir /build \
		advancedtelematic/ota-plus-client:latest \
		pkg/pkg.sh deb $(CURDIR)

rpm: image ## Make a new RPM package inside a Docker container.
	@docker run --rm \
		--env CARGO_HOME=/cargo \
		--volume ~/.cargo:/cargo \
		--volume $(CURDIR):/build \
		--workdir /build \
		advancedtelematic/ota-plus-client:latest \
		pkg/pkg.sh deb $(CURDIR)
