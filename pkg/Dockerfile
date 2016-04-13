FROM clux/muslrust
# for statically linking binaries; see https://github.com/clux/muslrust

ENV FPM_VER 1.5.0

RUN apt-get update \
  && apt-get install -y \
    devscripts \
    dh-systemd \
    openssl \
    patch \
    rpm \
    ruby \
    ruby-dev \
  && rm -rf /var/lib/apt/lists/*

# install fpm
RUN gem install fpm -v ${FPM_VER}
# apply patch to add --rpm-service flag
COPY pr862.patch /
RUN patch -i /pr862.patch /var/lib/gems/2.1.0/gems/fpm-${FPM_VER}/lib/fpm/package/rpm.rb

WORKDIR /build
CMD ["make", "all"]
