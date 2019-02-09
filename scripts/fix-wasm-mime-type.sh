#!/bin/bash
set -euxo pipefail

# When travis-ci deploys wasm files to s3, they end up with no mime type.
# Wasm files must have the mime type "application/wasm" to work properly.
# This script downloads latest wasm binary for this project and re-uploads
# it with the correct mime type.

APP_NAME="$1"
VERSION=$2
URL="s3://games.gridbugs.org/$APP_NAME/$VERSION/app.wasm"

TMP=$(mktemp -d)
trap "rm -rf $TMP" EXIT

WASM_PATH=$TMP/app.wasm

s3cmd get $URL $WASM_PATH

s3cmd --mime-type="application/wasm" --no-mime-magic --no-guess-mime-type \
    put $WASM_PATH $URL
