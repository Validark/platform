#!/usr/bin/env bash

set -e

SCRIPT_PATH=$(realpath "$0")
SCRIPT_DIRECTORY_PATH=$(dirname "$SCRIPT_PATH")
PROJECT_ROOT_PATH=$(dirname "$SCRIPT_DIRECTORY_PATH")
PACKAGES_PATH="$PROJECT_ROOT_PATH/packages"
LOGS_PATH="$PROJECT_ROOT_PATH/logs"

CONFIG=local
DAPI_PATH="${PACKAGES_PATH}"/dapi
DRIVE_PATH="${PACKAGES_PATH}"/rs-drive-abci
SDK_PATH="${PACKAGES_PATH}"/js-dash-sdk
RS_SDK_PATH="${PACKAGES_PATH}"/rs-sdk
WALLET_LIB_PATH="${PACKAGES_PATH}"/wallet-lib

# Get configuration option from dashmate config for 1st node
# Usage:
#   get_config <option>
#
# Example:
#   get_config core.rpc.port
function get_config() {
    # We use `jq` because it's much faster than `yarn dashmate config get`
    jq ".configs.${CONFIG}_1.$1" <"${HOME}"/.dashmate/config.json
}

touch "${LOGS_PATH}"/mint.log

# DAPI:
cp "${DAPI_PATH}"/.env.example "${DAPI_PATH}"/.env

# JS-SDK:
FAUCET_ADDRESS=$(grep -m 1 "Address:" "${LOGS_PATH}"/mint.log | awk '{printf $3}')
FAUCET_PRIVATE_KEY=$(grep -m 1 "Private key:" "${LOGS_PATH}"/mint.log | awk '{printf $4}')
# TODO This will be removed from dashmate. Please use hardcoded ID from contract crates
DPNS_CONTRACT_ID=$(get_config platform.dpns.contract.id)

SDK_ENV_FILE_PATH=${SDK_PATH}/.env
rm -f "${SDK_ENV_FILE_PATH}"
touch "${SDK_ENV_FILE_PATH}"

#cat << 'EOF' >> ${SDK_ENV_FILE_PATH}
echo "DAPI_SEED=127.0.0.1:2443:self-signed
FAUCET_ADDRESS=${FAUCET_ADDRESS}
FAUCET_PRIVATE_KEY=${FAUCET_PRIVATE_KEY}
DPNS_CONTRACT_ID=${DPNS_CONTRACT_ID}
NETWORK=regtest" >>"${SDK_ENV_FILE_PATH}"
#EOF

# DRIVE:
cp "${DRIVE_PATH}"/.env.example "${DRIVE_PATH}"/.env

# WALLET-LIB:
WALLET_LIB_ENV_FILE_PATH=${WALLET_LIB_PATH}/.env
rm -f "${WALLET_LIB_ENV_FILE_PATH}"
touch "${WALLET_LIB_ENV_FILE_PATH}"

#cat << 'EOF' >> ${SDK_ENV_FILE_PATH}
echo "DAPI_SEED=127.0.0.1:2443:self-signed
FAUCET_PRIVATE_KEY=${FAUCET_PRIVATE_KEY}
NETWORK=regtest" >>"${WALLET_LIB_ENV_FILE_PATH}"
#EOF

# RS_SDK tests config

CORE_RPC_PORT=$(get_config core.rpc.port)
CORE_RPC_USER=$(get_config core.rpc.user)
CORE_RPC_PASSWORD=$(get_config core.rpc.password)
PLATFORM_RPC_PORT=$(get_config platform.dapi.envoy.http.port)

cat <<EOF >"${RS_SDK_PATH}"/tests/.env
# Configuration of rs-sdk network tests
# Generated by configure_dotenv.sh

RS_SDK_PLATFORM_HOST="127.0.0.1"
RS_SDK_PLATFORM_PORT="$PLATFORM_RPC_PORT"

RS_SDK_CORE_PORT="$CORE_RPC_PORT"
RS_SDK_CORE_USER="$CORE_RPC_USER"
RS_SDK_CORE_PASSWORD="$CORE_RPC_PASSWORD"

EOF
