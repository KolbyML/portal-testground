#!/bin/bash

# Immediately abort the script on any error encountered
set -e

fluffy --rpc --rpc-address="0.0.0.0" --nat:extip:"$IP_ADDR" --network=none --log-level="debug"
