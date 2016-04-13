#!/bin/bash

set -eo pipefail

PKG_NAME="ota-plus-client"
PKG_VER=$VERSION
PKG_DIR="${PKG_NAME}-${PKG_VER}"
PKG_TARBALL="${PKG_NAME}_${PKG_VER}"
PREFIX=/opt/ats

cd $(dirname $0)
PKG_SRC_DIR=$(pwd)

function envsub {
  awk '
/^\s*(#.*)?$/ {
  print $0
  next
}{
  count = split($0, parts, /\${/)
  line = parts[1]
  for (i = 2; i <= count; i++) {
    if (split(parts[i], names, /}/) != 2)
      exit 1
    line = line""ENVIRON[names[1]]""names[2]
  }
  print line
}' $*
}

function make_deb {
  export PACKAGE_MANAGER="dpkg"

  cd $PKG_SRC_DIR
  envsub ota.toml.template > $PKG_NAME.toml

  fpm -s dir -t deb -n ${PKG_NAME} -v ${PKG_VER} --prefix ${PREFIX} -a native \
      --deb-systemd $PKG_SRC_DIR/ota-client.service \
      $PKG_SRC_DIR/ota_plus_client=ota_plus_client $PKG_NAME.toml=ota.toml

  mv -n ota-plus-client*.deb $dest
  rm $PKG_NAME.toml
}

function make_rpm {
  export PACKAGE_MANAGER="rpm"

  cd $PKG_SRC_DIR
  envsub ota.toml.template > $PKG_NAME.toml

  fpm -s dir -t rpm -n ${PKG_NAME} -v ${PKG_VER} --prefix ${PREFIX} -a native \
    --rpm-service $PKG_SRC_DIR/ota-client.service \
    $PKG_SRC_DIR/ota_plus_client=ota_plus_client $PKG_NAME.toml=ota.toml

  mv -n ota-plus-client*.rpm $dest
  rm $PKG_NAME.toml
}

if [ $# -lt 2 ]; then
  echo "Usage: $0 <package> <destination>"
  echo "packages: deb rpm"
  exit 1
fi

package="${1}"
dest="${2}"

echo "Building pkg to '$dest'"
case $package in
  "deb" )
    make_deb
    ;;
  "rpm" )
    make_rpm
    ;;
  *)
    echo "unknown package $package"
    exit 2
esac

