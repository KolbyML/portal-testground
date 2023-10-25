#!/bin/bash

# Immediately abort the script on any error encountered
set -e

DEBUG=* node /ultralight/packages/cli/dist/index.js --bindAddress="$IP_ADDR:9000" --dataDir="./data" --rpcAddr="0.0.0.0" --rpcPort=8545
