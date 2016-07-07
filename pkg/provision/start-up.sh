#!/bin/bash

set -eo pipefail

export OTA_AUTH_URL=${OTA_AUTH_URL-http://localhost:9001}
export OTA_WEB_URL=${OTA_WEB_URL-http://localhost:9000}
export OTA_CORE_URL=${OTA_CORE_URL-http://localhost:8080}
export PACKAGE_MANAGER=${PACKAGE_MANAGER-'dpkg'}
export OTA_WEB_USER="${OTA_WEB_USER-demo@advancedtelematic.com}"
export OTA_WEB_PASSWORD="${OTA_WEB_PASSWORD-demo}"
export OTA_HTTP=${OTA_HTTP-false}

if [[ -n $PROVISION ]]; then
  export OTA_CREDENTIALS_FILE=${OTA_CREDENTIALS_FILE-credentials.toml}
else
  export OTA_CREDENTIALS_FILE=${OTA_CREDENTIALS_FILE-/opt/ats/credentials.toml}
fi

TEMPLATE_PATH=${TEMPLATE_PATH-'/etc/ota.toml.template'}
AUTH_JSON_PATH=${AUTH_JSON_PATH-'/etc/auth.json'}
OUTPUT_PATH=${OUTPUT_PATH-/etc/ota.toml}

OTA_AUTH_PATH="/clients"
DEVICES_PATH="/api/v1/devices"

# Generate VIN
VIN_SUFFIX=$(< /dev/urandom tr -dc A-HJ-NPR-Z0-9 | head -c 11;echo;)
RANDOM_VIN=STRESS$VIN_SUFFIX
export OTA_CLIENT_VIN=${OTA_CLIENT_VIN-$RANDOM_VIN}

# Get cookie
HTTP_SESSION="/tmp/$OTA_CLIENT_VIN.json"
http --check-status --session=$HTTP_SESSION POST ${OTA_WEB_URL}/authenticate \
     username=$OTA_WEB_USER password=$OTA_WEB_PASSWORD --ignore-stdin || [[ $? == 3 ]]

# Add device to ota-plus web
export OTA_CLIENT_UUID=$(http --check-status --ignore-stdin --session=$HTTP_SESSION ${OTA_WEB_URL}${DEVICES_PATH} deviceName=${OTA_CLIENT_VIN} deviceId=${OTA_CLIENT_VIN} deviceType=Vehicle | cut -c2-37)
echo "created device $OTA_CLIENT_UUID"

# Get VIN credentials
JSON=$(envsubst < $AUTH_JSON_PATH)
AUTH_DATA=$(echo $JSON | http --check-status post $OTA_AUTH_URL$OTA_AUTH_PATH)
CLIENT_ID=$(echo $AUTH_DATA | jq -r .client_id)
SECRET=$(echo $AUTH_DATA | jq -r .client_secret)

export OTA_AUTH_CLIENT_ID=${OTA_AUTH_CLIENT_ID-$CLIENT_ID}
export OTA_AUTH_SECRET=${OTA_AUTH_SECRET-$SECRET}

if [[ -n $PROVISION ]]; then
  OTA_TOML=$(cat $TEMPLATE_PATH | envsubst )
  echo "$OTA_TOML"
else
  OTA_TOML=$(cat $TEMPLATE_PATH | envsubst > $OUTPUT_PATH)
  cat $OUTPUT_PATH
  RUST_LOG=${RUST_LOG-debug} ota_plus_client --config=/etc/ota.toml
fi
