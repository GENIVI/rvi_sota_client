#!/bin/bash

set -eo pipefail

export OTA_WEB_URL=${OTA_WEB_URL-http://ota-plus-web-staging.gw.prod01.advancedtelematic.com}
export OTA_CORE_URL=${OTA_CORE_URL-http://ota-plus-core-staging.gw.prod01.advancedtelematic.com}
export OTA_AUTH_URL=${OTA_AUTH_URL-http://auth-plus-staging.gw.prod01.advancedtelematic.com}

OTA_AUTH_PATH="/clients"

VEHICLES_PATH="/api/v1/vehicles/"

PACKAGE_MANAGER=${PACKAGE_MANAGER-'dpkg'}

TEMPLATE_PATH=${TEMPLATE_PATH-'/etc/ota.toml.template'}

VIN_SUFFIX=$(< /dev/urandom tr -dc A-HJ-NPR-Z0-9 | head -c 11;echo;)

echo $VIN_SUFFIX
export RANDOM_VIN=STRESS$VIN_SUFFIX
export OTA_CLIENT_VIN=${OTA_CLIENT_VIN-$RANDOM_VIN}
export HTTP_SESSION="/tmp/$OTA_CLIENT_VIN.json"
export OTA_WEB_USER="${OTA_WEB_USER-demo@advancedtelematic.com}"
export OTA_WEB_PASSWORD="${OTA_WEB_PASSWORD-demo}"

#export OTA_CLIENT_VIN=STRESS12345678901

http --check-status --session=$HTTP_SESSION POST ${OTA_WEB_URL}/authenticate \
     username=$OTA_WEB_USER password=$OTA_WEB_PASSWORD --ignore-stdin || [[ $? == 3 ]]

echo "vin=${OTA_CLIENT_VIN}" | http --check-status --session=$HTTP_SESSION put "${OTA_WEB_URL}${VEHICLES_PATH}${OTA_CLIENT_VIN}"
AUTH_JSON_PATH=${AUTH_JSON_PATH-'/etc/auth.json'}
JSON=$(envsubst < $AUTH_JSON_PATH)
AUTH_DATA=$(echo $JSON | http --check-status post $OTA_AUTH_URL$OTA_AUTH_PATH)

CLIENT_ID=$(echo $AUTH_DATA | jq -r .client_id)
SECRET=$(echo $AUTH_DATA | jq -r .client_secret)

export OTA_CLIENT_VIN=$OTA_CLIENT_VIN
export OTA_AUTH_URL=$OTA_AUTH_URL
export OTA_CORE_URL=$OTA_CORE_URL
export OTA_AUTH_CLIENT_ID=${OTA_AUTH_CLIENT_ID-$CLIENT_ID}
export OTA_AUTH_SECRET=${OTA_AUTH_SECRET-$SECRET}
export PACKAGE_MANAGER=$PACKAGE_MANAGER
export OTA_HTTP=${OTA_HTTP-false}

echo $OTA_CLIENT_VIN
echo $OTA_AUTH_URL
echo $OTA_CORE_URL
echo $OTA_AUTH_CLIENT_ID
echo $OTA_AUTH_SECRET
export $PACKAGE_MANAGER

OUTPUT_PATH=${OUTPUT_PATH-/etc/ota.toml}

while getopts ":p" opt; do
  PROVISION='false'
  case $opt in
    p)
      PROVISION='true'
      ;;
  esac
done

if [[ $PROVISION == 'true' ]]
then
  OTA_TOML=$(cat $TEMPLATE_PATH | envsubst )
  echo "$OTA_TOML"
else
  OTA_TOML=$(cat $TEMPLATE_PATH | envsubst > $OUTPUT_PATH)
  cat $OUTPUT_PATH
  RUST_LOG=debug ota_plus_client --config=/etc/ota.toml
fi
