FROM debian:8.4

RUN apt-get update

EXPOSE 8080

RUN apt-get install -y httpie jq gettext

COPY ota_plus_client /usr/bin/
COPY ota.toml.template /etc/
COPY provision/start-up.sh /usr/bin/
COPY provision/auth.json /etc/

ENV LANG="en_GB.UTF-8"

CMD start-up.sh
