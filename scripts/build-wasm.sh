#!/bin/bash
set -euxo pipefail

APP_NAME=$1

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
ROOT_DIR=$DIR/..
WEB_UPLOADS=$ROOT_DIR/web_uploads
WASM_CRATE=$ROOT_DIR/wasm
TARGET=$ROOT_DIR/target
BUILD_WASM=$DIR/build-wasm.py

source $DIR/build-common.sh

rm -rvf $WEB_UPLOADS
mkdir -vp $WEB_UPLOADS

pushd $WASM_CRATE
npm install
popd

BINARYEN_MD5="cfc00c1f1a0c05c4c40a8bb222439c11"
BINARYEN_URL="https://github.com/WebAssembly/binaryen/releases/download/1.38.26/binaryen-1.38.26-x86-linux.tar.gz"
BINARYEN_DIR="$ROOT_DIR/binaryen-1.38.26"
WASM2JS=$BINARYEN_DIR/wasm2js
if [[ ! -e $WASM2JS || $(md5sum $WASM2JS | cut -d' ' -f1) != $BINARYEN_MD5 ]]; then
  curl -sSL $BINARYEN_URL -o - | tar xzv
fi

$PYTHON $BUILD_WASM --manifest-path=$WASM_CRATE/Cargo.toml --webapp-dir=$WASM_CRATE \
  --target-dir=$TARGET --output-dir=$WEB_UPLOADS/$APP_NAME-js --wasm2js=$WASM2JS \
  --branch=$TRAVIS_BRANCH --release
  rm -rvf $WASM_CRATE/wasm_out
  $PYTHON $BUILD_WASM --manifest-path=$WASM_CRATE/Cargo.toml --webapp-dir=$WASM_CRATE \
    --target-dir=$TARGET --output-dir=$WEB_UPLOADS/$APP_NAME --branch=$TRAVIS_BRANCH --release
