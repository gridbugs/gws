#!/bin/bash
set -euxo pipefail

APP_NAME=$1
MACOS_APP_NAME=$2

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
ROOT_DIR=$DIR/..
UPLOADS=$ROOT_DIR/uploads
UNIX_CRATE=$ROOT_DIR/unix
GLUTIN_CRATE=$ROOT_DIR/glutin
TARGET=$ROOT_DIR/target
BUILD_NATIVE=$DIR/build-native.py
PACKAGE=$ROOT_DIR/package

source $DIR/build-common.sh

rm -rvf $UPLOADS
mkdir -vp $UPLOADS

case $TRAVIS_OS_NAME in
    linux)
        $PYTHON $BUILD_NATIVE --root-dir=$ROOT_DIR --unix-path=$UNIX_CRATE/Cargo.toml --glutin-path=$GLUTIN_CRATE/Cargo.toml \
            --target-dir=$TARGET --output-dir=$UPLOADS --name=$APP_NAME --os=linux --branch=$TRAVIS_BRANCH \
            --package-path=$PACKAGE --release
        ;;
    osx)
        $PYTHON $BUILD_NATIVE --root-dir=$ROOT_DIR --unix-path=$UNIX_CRATE/Cargo.toml --glutin-path=$GLUTIN_CRATE/Cargo.toml \
            --target-dir=$TARGET --output-dir=$UPLOADS --name=$APP_NAME --os=macos --macos-app-name $MACOS_APP_NAME \
            --branch=$TRAVIS_BRANCH --package-path=$PACKAGE --release
        ;;
esac
