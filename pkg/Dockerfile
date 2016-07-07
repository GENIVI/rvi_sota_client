FROM advancedtelematic/fpm

COPY ota_plus_client /usr/bin/
COPY ota.toml.template /etc/
COPY start-up.sh /usr/bin/

ENV LANG="en_US.UTF-8"
WORKDIR /build
EXPOSE 8888

CMD start-up.sh
