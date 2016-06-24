# sota-client

This is the client (in-vehicle) portion of the SOTA project. See the [main SOTA Server project](https://github.com/advancedtelematic/rvi_sota_server) and [associated architecture document](http://advancedtelematic.github.io/rvi_sota_server/dev/architecture.html) for more information.

## Building and running

To see the SOTA client in action, you will need to first build and run the [SOTA Core Server](https://github.com/advancedtelematic/rvi_sota_server).

### Building SOTA Client

As long as you have `rust 1.8.0` and `cargo` installed, `cargo build` should build the `sota_client` executable in `target/debug`.

### Running SOTA Client

You can run the client with `target/debug/sota_client -c client.toml`. It will try to connect to the `core` service of `rvi_sota_server` specified in the `[server]` section of `client.toml`. If the `[server.auth]` section contains `client_id`, `client_secret` and `url`, it will first try to obtain an OAuth access token from `url` and then authenticate all the requests to the server with it.

#### Running with HTTP communication

HTTP tends to be much easier to get working than RVI. Enable HTTP by setting `http = “true”` in the [client] section of `client.toml`, and setting the url value in the [server] section to the url of your sota-core deployment. 

#### Running with RVI nodes

To connect to the SOTA Server over RVI, run the `rvi_sota_server` project with RVI Nodes.

You can build RVI directly from [its GitHub repo](https://github.com/GENIVI/rvi_core), or simply run our docker image. These instructions assume you are running the docker image.

1. Pull the image: `docker pull advancedtelematic/rvi`.
2. In two terminal windows, run the rvi client and server nodes
  * Client: `docker run -it --name rvi-client --expose 8901 --expose 8905-8908 -p 8901:8901 -p 8905:8905 -p 8906:8906 -p 8907:8907 -p 8908:8908 advancedtelematic/rvi client`
  * Server: `docker run -it --name rvi-server --expose 8801 --expose 8805-8808 -p 8801:8801 -p 8805:8805 -p 8806:8806 -p 8807:8807 -p 8808:8808 advancedtelematic/rvi server`

Now you can remove the `[server]` section from `client.toml` and disable http.

### Running with GENIVI Software Loading Manager

You can run the (GENIVI SWLM)[https://github.com/GENIVI/genivi_swm] to process the incoming update. You will need to run both the SWLM and SOTA Client as root to communicate over the DBus session.

### Documentation

To create a static HTML version of the module documentation run `cargo doc`.
Unfortunately this will only create documentation for the public interface. If
you want the full documentation you need to run `cargo doc -v` extract the
`rustdoc` command from the output and append `--no-defaults --passes
"collapse-docs" --passes "unindent-comments" --passes strip-hidden` to it.
