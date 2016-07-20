#!/bin/bash

set -eo pipefail

# set default environment variables
AUTH_SECTION="${AUTH_SECTION:-false}"
AUTH_PLUS_URL="${AUTH_PLUS_URL:-http://localhost:9001}"
DEVICE_REGISTRY_URL="${DEVICE_REGISTRY_URL:-http://localhost:8083}"
TEMPLATE_PATH="${TEMPLATE_PATH:-/etc/sota.toml.template}"
OUTPUT_PATH="${OUTPUT_PATH:-/etc/sota.toml}"

# generate device vin
RAND=$(< /dev/urandom tr -dc A-HJ-NPR-Z0-9 | head -c 13 || [[ $? -eq 141 ]])
export DEVICE_VIN=${DEVICE_VIN:-"TEST${RAND}"}

# generate device uuid
if [[ -z "${DEVICE_UUID}" ]]; then
    DEVICE_UUID=$(http post "${DEVICE_REGISTRY_URL}/api/v1/devices" \
                       deviceName="${DEVICE_VIN}" \
                       deviceId="${DEVICE_VIN}" \
                       deviceType=Vehicle \
                       --check-status --print=b \
                      | tr -d '"')
fi
export DEVICE_UUID

# generate credentials
CREDENTIALS=$(http post "${AUTH_PLUS_URL}/clients" \
                   client_name="${DEVICE_VIN}" \
                   grant_types:='["client_credentials"]' \
                   --check-status --print=b)
export AUTH_CLIENT_ID=$(echo "${CREDENTIALS}" | jq -r .client_id)
export AUTH_SECRET=$(echo "${CREDENTIALS}" | jq -r .client_secret)

# optionally remove auth section
[[ "${AUTH_SECTION}" = false ]] && sed -i '1,/\[core\]/{/\[core\]/p;d}' "${TEMPLATE_PATH}"

# generate sota.toml config
[[ "${CONFIG_ONLY}" = true ]] && { envsubst < "${TEMPLATE_PATH}"; exit 0; }
envsubst < "${TEMPLATE_PATH}" | tee "${OUTPUT_PATH}"

# set up dbus
eval "$(dbus-launch)"
export DBUS_SESSION_BUS_ADDRESS
export DBUS_SESSION_BUS_PID

# start client
RUST_LOG="${RUST_LOG:-debug}" sota_client --config="${OUTPUT_PATH}"
