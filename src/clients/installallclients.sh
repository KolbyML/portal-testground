#! /bin/bash

# Immediately abort the script on any error encountered
set -e

export client_type='trin'
./getdocker.sh
export client_type='fluffy'
./getdocker.sh
export client_type='ultralight'
./getdocker.sh
