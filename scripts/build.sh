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

if [ -z ${TRAVIS_BRANCH+x} ]; then
    TRAVIS_BRANCH=$(git rev-parse --abbrev-ref HEAD)
fi

case $TRAVIS_OS_NAME in
    osx)
        if ! which python3 > /dev/null; then
            brew install python3 || brew upgrade python
        fi
        ;;
esac

PIP=pip3
PYTHON=python3

$PIP install --quiet --user sh toml

rm -rvf $UPLOADS
rm -rvf $WEB_UPLOADS
mkdir -vp $UPLOADS
mkdir -vp $WEB_UPLOADS

wasm_build() {
        pushd $WASM_CRATE
        npm install
        popd
        BINARYEN_URL="https://github.com/WebAssembly/binaryen/releases/download/1.38.26/binaryen-1.38.26-x86-linux.tar.gz"
        BINARYEN_DIR="binaryen-1.38.26"
        curl -sSL $BINARYEN_URL -o - | tar xzv
        WASM2JS=$BINARYEN_DIR/wasm2js
        $PYTHON $BUILD_WASM --manifest-path=$WASM_CRATE/Cargo.toml --webapp-dir=$WASM_CRATE \
            --target-dir=$TARGET --output-dir=$WEB_UPLOADS/$APP_NAME --branch=$TRAVIS_BRANCH --release
        $PYTHON $BUILD_WASM --manifest-path=$WASM_CRATE/Cargo.toml --webapp-dir=$WASM_CRATE \
            --target-dir=$TARGET --output-dir=$WEB_UPLOADS/$APP_NAME-js --wasm2js=$WASM2JS \
            --branch=$TRAVIS_BRANCH --release
        rm -rvf $BINARYEN_DIR
}

case $TRAVIS_OS_NAME in
    linux)
        wasm_build
        $PYTHON $BUILD_NATIVE --root-dir=$ROOT_DIR --unix-path=$UNIX_CRATE/Cargo.toml --glutin-path=$GLUTIN_CRATE/Cargo.toml \
            --target-dir=$TARGET --output-dir=$UPLOADS --name=$APP_NAME --os=linux --branch=$TRAVIS_BRANCH --release
        ;;
    osx)
        $PYTHON $BUILD_NATIVE --root-dir=$ROOT_DIR --unix-path=$UNIX_CRATE/Cargo.toml --glutin-path=$GLUTIN_CRATE/Cargo.toml \
            --target-dir=$TARGET --output-dir=$UPLOADS --name=$APP_NAME --os=macos --macos-app-name $MACOS_APP_NAME \
            --branch=$TRAVIS_BRANCH --release
        ;;
esac
