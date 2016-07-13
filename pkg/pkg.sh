#!/bin/bash

set -eo pipefail

if [ $# -lt 1 ]; then
  echo "Usage: $0 <package> [<destination>]"
  echo "packages: deb rpm"
  exit 1
fi

case $1 in
  "deb" )
    export PACKAGE_MANAGER="deb"
    PKG_BUILD_OPTS="--deb-systemd"
    ;;
  "rpm" )
    export PACKAGE_MANAGER="rpm"
    PKG_BUILD_OPTS=""
    ;;
  *)
    echo "unknown package format $1"
    exit 2
esac
shift

: ${PACKAGE_VERSION?"Environment variable PACKAGE_VERSION must be set."}

PKG_NAME=${PACKAGE_NAME-ota-plus-client}
PKG_SRC_DIR="$(cd "$(dirname "$0")" && pwd)"
PREFIX=/opt/ats

export OTA_CREDENTIALS_FILE=${OTA_CREDENTIALS_FILE-${PREFIX}/credentials.toml}
export OTA_HTTP=false

function make_pkg {
  dest=$1

  cfgfile=/tmp/$PKG_NAME.toml.$$
  envsubst < $PKG_SRC_DIR/ota.toml.template > $cfgfile
  chmod 600 $cfgfile

  fpm -s dir -t ${PACKAGE_MANAGER} -n ${PKG_NAME} -v ${PACKAGE_VERSION} --prefix ${PREFIX} \
    -p NAME-VERSION.TYPE -a native ${PKG_BUILD_OPTS} $PKG_SRC_DIR/ota-client.service \
    $PKG_SRC_DIR/ota_plus_client=ota_plus_client $cfgfile=ota.toml

  if [ -n "$dest" ]; then
    mv -f ota-plus-client*.${PACKAGE_MANAGER} $dest
  fi
  rm -f $cfgfile
}

make_pkg $*
