# OTA+ client

The OTA+ client source repository.

## Prerequisites

* Rust stable
* Cargo

## Build instructions

To build and test the project simply issue:

    cargo build
    cargo test

## Packaging instructions

A Dockerfile has been set up with the correct libraries for building a statically linked binary. This can be built from the project root with `docker build -t advancedtelematic/client-packager pkg`.

### DEB

A `.deb` package can be built with `docker run -e VERSION=0.0.0 -v $PWD:/build advancedtelematic/client-packager make deb` (or simply `make deb` with the correct build packages installed). Remember to set the `VERSION` environment variable to the correct version.

### RPM

[FPM](https://github.com/jordansissel/fpm) is used to create RPM packages.

A `.rpm` package can be built with `docker run -e VERSION=0.0.0 -v $PWD:/build advancedtelematic/client-packager make rpm` (or simply `make rpm` with FPM and the build packages installed). Remember to set the `VERSION` environment variable to the correct version.
