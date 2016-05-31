#!/bin/bash

set -eo pipefail

OTA_AUTH_PATH="/clients"

OTA_SERVER_PATH="/api/v1/vehicles/"

PACKAGE_MANAGER="dpkg"

TEMPLATE_PATH="/etc/ota.toml.template"

VIN_SUFFIX=$(< /dev/urandom tr -dc A-HJ-NPR-Z0-9 | head -c${1:-11};echo;)

echo $VIN_SUFFIX
export OTA_CLIENT_VIN=STRESS$VIN_SUFFIX
#export OTA_CLIENT_VIN=STRESS12345678901

echo "vin=${OTA_CLIENT_VIN}" | http put "${OTA_SERVER_URL}${OTA_SERVER_PATH}${OTA_CLIENT_VIN}"
JSON=$(envsubst < /etc/auth.json)
AUTH_DATA=$(echo $JSON | http post $OTA_AUTH_URL$OTA_AUTH_PATH)

OTA_AUTH_CLIENT_ID=$(echo $AUTH_DATA | jq -r .client_id)
OTA_AUTH_SECRET=$(echo $AUTH_DATA | jq -r .client_secret)

export OTA_CLIENT_VIN=$OTA_CLIENT_VIN
export OTA_AUTH_URL=$OTA_AUTH_URL
export OTA_SERVER_URL=$OTA_CORE_URL
export OTA_AUTH_CLIENT_ID=$OTA_AUTH_CLIENT_ID
export OTA_AUTH_SECRET=$OTA_AUTH_SECRET
export PACKAGE_MANAGER=$PACKAGE_MANAGER

echo $OTA_CLIENT_VIN
echo $OTA_AUTH_URL
echo $OTA_SERVER_URL
echo $OTA_AUTH_CLIENT_ID
echo $OTA_AUTH_SECRET
export $PACKAGE_MANAGER

OTA_TOML=$(cat $TEMPLATE_PATH | envsubst > /etc/ota.toml)
sed '/credentials_file/d' /etc/ota.toml
echo /etc/ota.toml

RUST_LOG=debug ota_plus_client --config=/etc/ota.toml
