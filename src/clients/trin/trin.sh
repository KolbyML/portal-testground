#!/bin/bash

# Immediately abort the script on any error encountered
set -e

RUST_LOG=debug trin --web3-transport http --web3-http-address http://0.0.0.0:8545 --external-address "$IP_ADDR":9009 --bootnodes none
