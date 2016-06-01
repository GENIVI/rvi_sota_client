FROM clux/muslrust

RUN apt-get update && apt-get install -y \
    devscripts \
    dh-systemd \
    gettext \
    httpie \
    jq \
    openssl \
    patch \
    rpm \
    ruby \
    ruby-dev \
  && rm -rf /var/lib/apt/lists/*

ENV FPM_VER 1.6.0

RUN gem install fpm -v ${FPM_VER}
# apply patch to add --rpm-service flag
COPY pr862.patch /
RUN patch -i /pr862.patch /var/lib/gems/2.1.0/gems/fpm-${FPM_VER}/lib/fpm/package/rpm.rb

COPY ota_plus_client /usr/bin/
COPY ota.toml.template /etc/
COPY provision/start-up.sh /usr/bin/
COPY provision/auth.json /etc/

ENV LANG="en_US.UTF-8"
WORKDIR /build
EXPOSE 8888

CMD start-up.sh
