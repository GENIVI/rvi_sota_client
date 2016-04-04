FROM clux/muslrust
# for statically linking binaries; see https://github.com/clux/muslrust

RUN apt-get update \
  && apt-get install -y \
    devscripts \
    dh-systemd \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /build
CMD ["make", "deb"]
