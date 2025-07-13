#!/bin/bash

BINARY_PATH="/opt/suzui-rs/suzui-rs"

while true; do
    # Countdown from 5 to 1
    for i in 5 4 3 2 1; do
        clear
        echo "Cycle Key IGN-ACC-IGN, Starting binary in $i seconds..."
        sleep 1
    done
    $BINARY_PATH
done
