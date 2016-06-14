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

## Dockerfile

There is a Dockerfile in `/pkg` to create and image with ota-plus-client that automatically configures itself with a random VIN. To build this image run:

```
make
docker build -t advancedtelematic/ota-plus-client pkg/
```

To use it, run:

```
docker run advancedtelematic/ota-plus-client
```

You can configure it using the following environment variables:

- `OTA_AUTH_URL`, default value: http://localhost:9001
- `OTA_WEB_URL`, default value: http://localhost:9000
- `OTA_CORE_URL`, default value: http://localhost:8080
- `OTA_WEB_USER`, default value: `demo@advancedtelematic.com`
- `OTA_WEB_PASSWORD`, default value: `demo`
- `OTA_CLIENT_VIN`, default value: Randomly generated
- `OTA_AUTH_CLIENT_ID`, default value: Generated for VIN
- `OTA_AUTH_SECRET`, default value: Generated for VIN
- `PACKAGE_MANAGER`, `dpkg` or `rpm`, default value: `dpkg`
- `OTA_HTTP`, default value: `false`
- `PROVISION`, default value: `false`. Set to `true` to output a configured `ota.toml` file to STDOUT then exit.

Eg:

```
docker run \
      -it \
      --rm \
      --net=host \
      -e OTA_AUTH_URL="http://auth-plus-staging.gw.prod01.advancedtelematic.com" \
      -e OTA_WEB_URL="http://ota-plus-web-staging.gw.prod01.advancedtelematic.com" \
      -e OTA_CORE_URL="http://ota-plus-core-staging.gw.prod01.advancedtelematic.com" \
      advancedtelematic/ota-plus-client:latest
```

If running against local urls, be sure to pass `--net=host` to the `docker run` command.
