# SOTA client

The client source repository for [Software Over The Air](http://advancedtelematic.github.io/rvi_sota_server/) updates.

## Prerequisites

The simplest way to get started is via [Docker](http://www.docker.com), which is used for compiling and running the client.

Alternatively (and optionally), local compilation requires a stable installation of Rust and Cargo. The easiest way to install both is via [Rustup](https://www.rustup.rs).

## Running the client

With Docker installed, `make run` will start the client.

### Makefile targets

Run `make help` to see the full list of targets, which are:

Target         | Description
-------------: | :----------
run            | Run the client inside a Docker container.
clean          | Remove all compiled libraries, builds and temporary files.
test           | Run all cargo tests.
doc            | Generate documentation for the sota crate.
clippy         | Run clippy lint checks using the nightly compiler.
client         | Compile a new release build of the client.
image          | Build a Docker image for running the client.
deb            | Create an installable DEB package of the client.
rpm            | Create an installable RPM package of the client.
version        | Print the version that will be used for building packages.

## Configuration

You can configure how the client starts with `make run` by setting the following environment variables:

Variable             | Default value             | Description
-------------------: | :------------------------ | :------------------
`AUTH_SECTION`       | `false`                   | Set to true to authenticate on startup.
`CONFIG_ONLY`        | `false`                   | Set to true to generate a config file then quit.
`AUTH_SERVER`        | http://127.0.0.1:9001     | The Auth server for client authentication.
`CORE_SERVER`        | http://127.0.0.1:9000     | The Core server for client communication.
`REGISTRY_SERVER`    | http://127.0.0.1:8083     | The server used for registering new devices.
`OUTPUT_PATH`        | `/etc/sota.toml`          | Path to write the newly generated config.
`TEMPLATE_PATH`      | `/etc/sota.toml.template` | Path to the template for new config files.
`DEVICE_VIN`         | (generated)               | Use this VIN rather than generating a new one.
`DEVICE_UUID`        | (generated)               | Use this UUID rather than generating a new one.
`AUTH_CLIENT_ID`     | (from registry server)    | Use this client ID for authentication.
`AUTH_CLIENT_SECRET` | (from registry server)    | Use this client secret for authentication.

For example, running `CONFIG_ONLY=true make run` will output a newly generated `sota.toml` to stdout then quit.

### Further customization

Every value in the generated `sota.toml` config file can be overwritten in the `run/sota.toml.env` file.

In addition, each config value is available as a command line flag when starting the client. Command line flags take precedence over the values set in the config file. Run `sota_client --help` to see a full list.

## Testing on GENIVI Development Platform over RVI

### Starting the SOTA Server

See the full documentation at [rvi_sota_server](http://advancedtelematic.github.io/rvi_sota_server/).

Here is a quickstart:

	git clone git@github.com:advancedtelematic/rvi_sota_server.git rvi_sota_server
	cd rvi_sota_server
	./sbt docker:publishLocal
	cd deploy/docker-compose
	docker-compose -f docker-compose.yml -f core-rvi.yml -f client-rvi.yml up -d

Login to the UI and create a new Device/Vehicle. Copy the newly generated Device UUID (e.g. "9ea653bc-3486-44cd-aa86-d936bd957e52") into the `client-rvi.yml` file as environment variable `DEVICE_ID`:

```
    environment:
      RVI_BACKEND: "rvi_backend"
      DEVICE_ID: "9ea653bc-3486-44cd-aa86-d936bd957e52"
```

Restart the RVI device node with the new DEVICE_ID by re-running:

	docker-compose -f docker-compose.yml -f core-rvi.yml -f client-rvi.yml up -d

### Configuring sota.toml

The `uuid` field in the `[device]` section must match the DEVICE_ID of the RVI node (e.g. "9ea653bc-3486-44cd-aa86-d936bd957e52").

The `rvi` and `dbus` fields in the `[gateway]` section must be `true`.

As the RVI device node is running inside a docker container (and thus cannot access 127.0.0.1 on the host), all URI fields should contain non-loopback IP addresses.

See `tests/genivi.sota.toml` for a sample config. See full documentation for details.

Now you can run the `sota_client`:

	make client
	RUST_LOG=debug ./run/sota_client --config tests/genivi.sota.toml

### GENIVI Software Loading Manager

See [genivi_swm](https://github.com/GENIVI/genivi_swm) on how to run the Software Loading Manager demo. It also contains instructions for creating an update image, which can be uploaded as a package to the SOTA Server.

Now you can create an update campaign on the SOTA Server, using the same update_id as the uuid in the update image you created. Also, as the genivi_swm demo runs as root, remember to run the `sota_client` as root as well so that they can communicate on the same system bus.
