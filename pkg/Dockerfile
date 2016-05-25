FROM clux/muslrust

RUN apt-get update && apt-get install -y \
    devscripts \
    dh-systemd \
    openssl \
    httpie \
    jq \
    gettext \
  && rm -rf /var/lib/apt/lists/*

COPY ota_plus_client /usr/bin/
COPY ota.toml.template /etc/
COPY provision/start-up.sh /usr/bin/
COPY provision/auth.json /etc/

ENV LANG="en_GB.UTF-8"
WORKDIR /build
EXPOSE 8888

CMD start-up.sh
