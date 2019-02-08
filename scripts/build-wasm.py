#!/usr/bin/env python

import argparse
import os
from os import path
import shutil
import sys
import toml
import sh

WASM_OUT = "wasm_out"
SH_KWARGS = {"_out": sys.stdout, "_err": sys.stderr}
TARGET = "wasm32-unknown-unknown"


def build_wasm(manifest_path, webapp_dir, target_dir, release, wasm2js):
    if release:
        cargo_args = ["--release"]
        mode = "release"
    else:
        cargo_args = []
        mode = "debug"
    sh.cargo.build(
        "--target", TARGET, "--manifest-path", manifest_path, *cargo_args, **SH_KWARGS
    )
    manifest = toml.load(manifest_path)
    crate_name = manifest["package"]["name"]
    binary_path = path.join(target_dir, TARGET, mode, "%s.wasm" % crate_name)
    wasm_out_path = path.join(webapp_dir, WASM_OUT)
    sh.mkdir("-p", wasm_out_path, **SH_KWARGS)
    sh.wasm_bindgen(
        binary_path, "--out-dir", wasm_out_path, "--out-name", "app", **SH_KWARGS
    )
    app_bg_wasm = path.join(wasm_out_path, "app_bg.wasm")
    app_bg_js = path.join(wasm_out_path, "app_bg.js")
    if wasm2js is not None:
        sh.Command(wasm2js)(app_bg_wasm, "-o", app_bg_js, **SH_KWARGS)
        sh.rm(path.join(app_bg_wasm), **SH_KWARGS)
    else:
        sh.rm("-f", app_bg_js, **SH_KWARGS)


def build_web_app(webapp_dir, release, output_dir):
    if release:
        webpack_mode = "production"
    else:
        webpack_mode = "development"
    e = {"WEBPACK_MODE": webpack_mode, "OUTPUT_DIR": output_dir}
    e.update(os.environ)
    sh.npx.webpack(_cwd=webapp_dir, _env=e, **SH_KWARGS)


def build(manifest_path, webapp_dir, target_dir, release, wasm2js, output_dir):
    build_wasm(manifest_path, webapp_dir, target_dir, release, wasm2js)
    if output_dir is not None:
        build_web_app(webapp_dir, release, output_dir)


def make_parser():
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest-path", default="wasm/Cargo.toml")
    parser.add_argument("--webapp-dir", default="wasm")
    parser.add_argument("--target-dir", default="target")
    parser.add_argument("--output-dir", required=False, type=str)
    parser.add_argument("--wasm2js", required=False, type=str)
    parser.add_argument("--release", action="store_true", default=False)
    return parser


def main(args):
    build(
        args.manifest_path,
        args.webapp_dir,
        args.target_dir,
        args.release,
        args.wasm2js,
        args.output_dir,
    )


if __name__ == "__main__":
    main(make_parser().parse_args())
