#!/bin/bash

set -eo pipefail

# FIXME(PRO-820): allow authentcation again via an environment variable

export OTA_CONSOLE=${OTA_CONSOLE-false}
export OTA_HTTP=${OTA_HTTP-false}
export OTA_WEBSOCKET=${OTA_WEBSOCKET-true}

export OTA_CORE_URL=${OTA_CORE_URL-http://localhost:8080}
export OTA_REGISTRY_URL=${OTA_REGISTRY_URL-http://localhost:8083}
export OTA_WEB_URL=${OTA_WEB_URL-http://localhost:9000}

export PACKAGE_MANAGER=${PACKAGE_MANAGER-'dpkg'}

# generate device ids
RANDOM_VIN="TEST$(< /dev/urandom tr -dc A-HJ-NPR-Z0-9 | head -c 13; echo;)"
export OTA_CLIENT_VIN=${OTA_CLIENT_VIN-$RANDOM_VIN}
RANDOM_UUID=$(http post "$OTA_REGISTRY_URL/api/v1/devices" \
  deviceType=Vehicle deviceName=$OTA_CLIENT_VIN deviceId=$OTA_CLIENT_VIN --print=b)
export OTA_CLIENT_UUID=$(echo $RANDOM_UUID | tr -d '"')

if [[ -n $PROVISION ]]; then
  export OTA_CREDENTIALS_FILE=${OTA_CREDENTIALS_FILE-credentials.toml}
else
  export OTA_CREDENTIALS_FILE=${OTA_CREDENTIALS_FILE-/opt/ats/credentials.toml}
fi

TEMPLATE_PATH=${TEMPLATE_PATH-'/etc/ota.toml.template'}
OUTPUT_PATH=${OUTPUT_PATH-/etc/ota.toml}


if [[ -n $PROVISION ]]; then
  OTA_TOML=$(cat $TEMPLATE_PATH | envsubst )
  echo "$OTA_TOML"
else
  OTA_TOML=$(cat $TEMPLATE_PATH | envsubst > $OUTPUT_PATH)
  cat $OUTPUT_PATH
  RUST_LOG=${RUST_LOG-debug} ota_plus_client --config=/etc/ota.toml
fi
