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

### DEB

A Dockerfile has been set up with the correct libraries for building a statically linked binary. This can be built from the project root with `docker build -t deb-packager pkg/deb`. The DEB package can then be built with `docker run -it --rm -v $PWD:/build deb-packager`.

Alternatively, with the correct build packages installed, `make deb` can be run from the project root.

### RPM

[FPM](https://github.com/jordansissel/fpm) is used to create RPM packages.

A Dockerfile has been set up with the correct libraries and can be built from the project root with `docker build -t rpm-packager pkg/rpm`. An RPM can then be created with `docker run -it --rm -v $PWD:/build rpm-packager`.

Alternatively, assuming FPM and the correct libraries are installed, an RPM can be built with `make rpm`.
