#!/usr/bin/env bash

set -e

MINING_INTERVAL_IN_SECONDS=30
MASTERNODES_COUNT=3

FULL_PATH=$(realpath $0)
DIR_PATH=$(dirname $FULL_PATH)
ROOT_PATH=$(dirname $DIR_PATH)

DASHMATE="$PATH_TO_PROJECT_ROOT"/scripts/dashmate

$DASHMATE setup local --verbose \
                                  --debug-logs \
                                  --miner-interval="${MINING_INTERVAL_IN_SECONDS}s" \
                                  --node-count=${MASTERNODES_COUNT} | tee "${ROOT_PATH}"/logs/setup.log || exit 1

# enable insight
$DASHMATE config set core.insight.enabled true --config local_seed
