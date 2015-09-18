#!/bin/bash
# bash "strict mode", see
# http://redsymbol.net/articles/unofficial-bash-strict-mode/
set -euo pipefail
IFS=$'\n\t'

# get the local dockers IP address
# only works if the RVI client is running on the same node host.
LINK="$(ip addr show eth0 | grep 'inet ' | awk '{ print $2 }' | sed 's,/.*$,,')"

LOGLEVEL=${LOGLEVEL:-"info"}

RUST_LOG=${RUST_LOG:-"sota_client=$LOGLEVEL"} \
  /bin/sota_client \
  "http://${RVI_ADDR:-rvi-client}:${RVI_PORT:-8901}" \
  "${SOTA_CLIENT_ADDR:-$LINK}:${SOTA_CLIENT_PORT:-9000}"
