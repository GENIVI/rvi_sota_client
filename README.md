# sota-client

This is the client (in-vehicle) portion of the SOTA project. It is provided as an RPM that can be installed on a target system. See the [main SOTA Server project](https://github.com/advancedtelematic/sota-server) and [associated architecture document](https://github.com/advancedtelematic/sota-server/wiki/Architecture) for more information.

## Build Docker container

A docker container can be built with

```
make docker
```

Note, that you need to have `docker`, `make`, `rust` and `cargo` installed for
this to work. If you want to change the tag that is applied to the image either
retag after every build with

```
docker tag advancedtelematic/sota-client:latest you/some-name:version
```

or edit the `Makefile` to your taste.

See `README.md` in the `docker` subfolder for instructions how to run the
container.
