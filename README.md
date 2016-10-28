# SOTA client

The client source repository for [Software Over The Air](http://advancedtelematic.github.io/rvi_sota_server/) updates.

Click [here](http://advancedtelematic.github.io/rvi_sota_server/cli/client-commands-and-events-reference.html) for the complete `Command` and `Event` API reference used for communicating with the client.

## Prerequisites

The simplest way to get started is via [Docker](http://www.docker.com), which is used for compiling and running the client.

Alternatively (and optionally), local compilation requires a stable installation of Rust and Cargo. The easiest way to install both is via [Rustup](https://www.rustup.rs).

## Configuration

See [here](http://advancedtelematic.github.io/rvi_sota_server/cli/client-startup-and-configuration.html) for full details on configuring the client on startup.

### Makefile configuration

Run `make help` to see the full list of targets.

With Docker installed, `make run` will start the client. You can configure how the client starts by setting the following environment variables:

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

Every value in the generated `sota.toml` config file can be overwritten in the `run/sota.toml.env` file. In addition, each config value is available as a command line flag when starting the client. Command line flags take precedence over the values set in the config file. Run `sota_client --help` to see a full list.

## GENIVI Development Platform over RVI

See the full documentation at [rvi_sota_server](http://advancedtelematic.github.io/rvi_sota_server/).

### Starting the SOTA Server

To get started:

    git clone https://github.com/advancedtelematic/rvi_sota_server rvi_sota_server
    cd rvi_sota_server
    ./sbt docker:publishLocal
    cd deploy/docker-compose
    docker-compose -f docker-compose.yml -f core-rvi.yml -f client-rvi.yml up -d

Log in to the UI and create a new Device/Vehicle. Click the device name to view it then copy the UUID from the URL (e.g. "9ea653bc-3486-44cd-aa86-d936bd957e52").

In the `client-rvi.yml` file, replace the `DEVICE_ID` environment variable with the UUID copied above. Now restart the RVI device node with:

	docker-compose -f docker-compose.yml -f core-rvi.yml -f client-rvi.yml up -d

### Configuring sota.toml

See `tests/toml/genivi.toml` for a sample config.

The `uuid` field in the `[device]` section must match the DEVICE_ID of the RVI node (e.g. "9ea653bc-3486-44cd-aa86-d936bd957e52"). In addition, set the `rvi` and `dbus` fields in the `[gateway]` section to `true`.

As the RVI device node is running inside a docker container (and thus cannot access 127.0.0.1 on the host), all URI fields should contain non-loopback IP addresses.

Now you can start the client:

	make client
	RUST_LOG=debug run/sota_client --config tests/toml/genivi.sota.toml

### GENIVI Software Loading Manager

See [genivi_swm](https://github.com/GENIVI/genivi_swm) for complete instructions on how to run the Software Loading Manager (SWM) demo, including instructions on creating a new update image to upload to the SOTA Server.

To get started:

    git clone https://github.com/GENIVI/genivi_swm
    cd genivi_swm
    export PYTHONPATH="${PWD}/common/"
    python software_loading_manager/software_loading_manager.py

As the genivi_swm demo runs as root, remember to run the `sota_client` as root as well so that they can communicate on the same system bus.

Now you can create an update campaign on the SOTA Server using the same update_id as the uuid in the update image you created.
