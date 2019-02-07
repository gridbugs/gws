#!/bin/bash
set -euxo pipefail

APP_NAME=$1
MACOS_APP_NAME=$2

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
ROOT_DIR=$DIR/..
WEB_UPLOADS=$ROOT_DIR/web_uploads
UPLOADS=$ROOT_DIR/uploads
UNIX_CRATE=$ROOT_DIR/unix
GLUTIN_CRATE=$ROOT_DIR/glutin
WASM_CRATE=$ROOT_DIR/wasm
TARGET=$ROOT_DIR/target
BUILD_NATIVE=$DIR/build-native.py
BUILD_WASM=$DIR/build-wasm.py

if [ -z ${TRAVIS_OS_NAME+x} ]; then
    case `uname -s` in
        Linux)
            TRAVIS_OS_NAME=linux
            ;;
        Darwin)
            TRAVIS_OS_NAME=osx
            ;;
        *)
            echo "Unknown OS"
            exit 1
    esac
fi

case $TRAVIS_OS_NAME in
    linux)
        PIP=pip
        PYTHON=python
        ;;
    osx)
        if ! which python3 > /dev/null; then
            brew install python3 || brew upgrade python
        fi
        PIP=pip3
        PYTHON=python3
        ;;
esac

$PIP install --quiet --user sh toml

rm -rf $UPLOADS
rm -rf $WEB_UPLOADS
mkdir -p $UPLOADS
mkdir -p $WEB_UPLOADS

wasm_build() {
        BINARYEN_URL="https://github.com/WebAssembly/binaryen/releases/download/1.38.26/binaryen-1.38.26-x86-linux.tar.gz"
        BINARYEN_DIR="binaryen-1.38.26"
        curl -sSL $BINARYEN_URL -o - | tar xzv
        WASM2JS=$BINARYEN_DIR/wasm2js
        $BUILD_WASM --manifest-path=$WASM_CRATE/Cargo.toml --webapp-dir=$WASM_CRATE \
            --target-dir=$TARGET --output-dir=$WEB_UPLOADS/$APP_NAME --release
        $BUILD_WASM --manifest-path=$WASM_CRATE/Cargo.toml --webapp-dir=$WASM_CRATE \
            --target-dir=$TARGET --output-dir=$WEB_UPLOADS/$APP_NAME-js --wasm2js=$WASM2JS --release
}

case $TRAVIS_OS_NAME in
    linux)
        wasm_build
        $BUILD_NATIVE --root-dir=$ROOT_DIR --unix-path=$UNIX_CRATE/Cargo.toml --glutin-path=$GLUTIN_CRATE/Cargo.toml \
            --target-dir=$TARGET --output-dir=$UPLOADS --name=$APP_NAME --os=linux --release
        ;;
    osx)
        $BUILD_NATIVE --root-dir=$ROOT_DIR --unix-path=$UNIX_CRATE/Cargo.toml --glutin-path=$GLUTIN_CRATE/Cargo.toml \
            --target-dir=$TARGET --output-dir=$UPLOADS --name=$APP_NAME --os=macos --release --macos-app-name $MACOS_APP_NAME
        ;;
esac
