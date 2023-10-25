#!/bin/bash

# Immediately abort the script on any error encountered
set -e

IP_ADDR=$(hostname -i | awk '{print $1}')

# if CLIENT_PRIVATE_KEY isn't set or doesn't exist do y, else do z
if [ -z ${CLIENT_PRIVATE_KEY+x} ]; then
  fluffy --rpc --rpc-address="0.0.0.0" --nat:extip:"$IP_ADDR" --network=none --log-level="debug"
else
  fluffy --rpc --rpc-address="0.0.0.0" --nat:extip:"$IP_ADDR" --network=none --log-level="debug" --netkey-unsafe=0x${CLIENT_PRIVATE_KEY}
fi