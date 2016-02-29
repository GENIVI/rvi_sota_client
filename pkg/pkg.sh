#!/bin/bash

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

if [ $# -lt 1 ]
then
  echo "Usage: $0 <DEST>"
  exit 1
fi

dest="${1}"
echo "Building pkg to '$dest'"

PKG_NAME="ota-plus-client"
PKG_VER="0.1.0"
PKG_DIR="${PKG_NAME}-${PKG_VER}"
PKG_TARBALL="${PKG_NAME}_${PKG_VER}"

cd `dirname $0`
PKG_SRC_DIR=`pwd`

workdir="${TMPDIR:-/tmp}/pkg-ota-plus-client-$$"
cp -pr deb $workdir
cd $workdir

mkdir -p $PKG_DIR/bin
mkdir -p $PKG_DIR/etc
envsub ${PKG_SRC_DIR}/ota.toml.template > $PKG_DIR/etc/ota.toml
tar czf $PKG_TARBALL $PKG_DIR/bin $PKG_DIR/etc

cd $PKG_DIR
debuild -i -us -uc -b
cd ..
cp ota-plus-client*.deb "${dest}"

cd $PKG_SRC_DIR
rm -rf $workdir
