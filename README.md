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
