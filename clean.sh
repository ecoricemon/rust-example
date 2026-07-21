#!/bin/bash

# Clean each Cargo project.
for dir in *; do
    if [ -d "$dir" ]; then
        echo "=== Cleaning $dir... ==="
        pushd .
        cd $dir
        cargo clean
        popd
    fi
done
echo "=== Done. ==="
