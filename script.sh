#!/bin/bash

BINARY_PATH="/opt/suzui-rs/suzui-rs"

while true; do
    $BINARY_PATH
    echo "Waiting 5 seconds before retrying"
    sleep 5
    clear
done
