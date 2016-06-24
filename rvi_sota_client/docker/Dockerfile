FROM debian:8

RUN apt-get update \
 && apt-get install -y openssl dbus libdbus-1-3 dbus-x11 libdbus-glib-1-2 \
 && mkdir /var/sota

COPY sota_client /usr/bin/sota_client
COPY run.sh /usr/bin/run.sh
COPY client.toml /var/sota/client.toml

EXPOSE 9080
CMD ["/usr/bin/run.sh"]
