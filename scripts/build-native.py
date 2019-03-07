#!/usr/bin/env python

import argparse
import os
from os import path
import shutil
import sys
import zipfile
import tempfile
import toml
import sh

SH_KWARGS = {"_out": sys.stdout, "_err": sys.stderr}
GLUTIN_SUFFIX = "graphical"
UNIX_SUFFIX = "terminal"
ARCHITECTURE = "x86_64"


def build_single(manifest_path, manifest, mode, cargo_mode_args, target_dir, dest):
    sh.cargo.build("--manifest-path", manifest_path, *cargo_mode_args, **SH_KWARGS)
    shutil.copy(path.join(target_dir, mode, manifest["package"]["name"]), dest)


def make_zip(base_name, stage_dir, dest_dir):
    zip_name = "%s.zip" % base_name
    zip_path = path.join(dest_dir, zip_name)
    with zipfile.ZipFile(zip_path, "w") as zip_file:
        for subdir, dirs, files in os.walk(stage_dir):
            for f in files:
                local_name = path.join(base_name, f)
                zip_file.write(path.join(subdir, f), local_name)


def make_dmg(dmg_name, dmg_dir, dest_dir):
    sh.hdiutil.create(path.join(dest_dir, dmg_name), "-srcfolder", dmg_dir)


def build(
    name,
    os_name,
    root_dir,
    unix_path,
    glutin_path,
    release,
    branch,
    target_dir,
    output_dir,
    macos_app_name,
    package_path,
):
    unix_manifest = toml.load(unix_path)
    glutin_manifest = toml.load(glutin_path)
    if unix_manifest["package"]["version"] != glutin_manifest["package"]["version"]:
        raise Exception("Version mismatch between unix and glutin crates")
    version = unix_manifest["package"]["version"]
    if release:
        cargo_mode_args = ["--release"]
        mode = "release"
    else:
        cargo_mode_args = []
        mode = "debug"
    stage_dir = tempfile.mkdtemp()
    sh.mkdir("-p", stage_dir, **SH_KWARGS)
    unix_bin_path = path.join(stage_dir, "%s-%s" % (name, UNIX_SUFFIX))
    build_single(
        unix_path, unix_manifest, mode, cargo_mode_args, target_dir, unix_bin_path
    )
    glutin_bin_path = path.join(stage_dir, "%s-%s" % (name, GLUTIN_SUFFIX))
    build_single(
        glutin_path, glutin_manifest, mode, cargo_mode_args, target_dir, glutin_bin_path
    )
    shutil.copy(path.join(root_dir, "LICENSE"), path.join(stage_dir, "LICENSE.txt"))
    shutil.copy(path.join(root_dir, "README.md"), path.join(stage_dir, "README.txt"))
    sh.git(
        "rev-parse", "HEAD", _err=sys.stdout, _out=path.join(stage_dir, "REVISION.txt")
    )
    version_number_name = "%(name)s-%(os)s-%(architecture)s-v%(version)s" % {
        "name": name,
        "os": os_name,
        "architecture": ARCHITECTURE,
        "version": version,
    }
    branch_name = "%(name)s-%(os)s-%(architecture)s-%(branch)s" % {
        "name": name,
        "os": os_name,
        "architecture": ARCHITECTURE,
        "branch": branch,
    }
    sh.mkdir("-p", output_dir, **SH_KWARGS)
    make_zip(version_number_name, stage_dir, output_dir)
    make_zip(branch_name, stage_dir, output_dir)
    if macos_app_name is not None:
        dmg_dir = path.join(stage_dir, macos_app_name)
        app_dir = path.join(dmg_dir, "%s.app" % macos_app_name)
        macos_dir = path.join(app_dir, "Contents", "MacOS")
        sh.mkdir("-p", macos_dir)
        shutil.copy(path.join(stage_dir, "LICENSE.txt"), dmg_dir)
        shutil.copy(path.join(stage_dir, "README.txt"), dmg_dir)
        shutil.copy(path.join(stage_dir, "REVISION.txt"), dmg_dir)
        shutil.copy(path.join(package_path, "macos-run-app.sh"), path.join(macos_dir, macos_app_name))
        shutil.copy(glutin_bin_path, path.join(macos_dir, "app"))
        os.symlink("/Applications", path.join(dmg_dir, "Applications"))
        version_number_dmg_name = "%s-v%s.dmg" % (macos_app_name, version)
        branch_dmg_name = "%s-%s.dmg" % (macos_app_name, branch)
        make_dmg(version_number_dmg_name, dmg_dir, output_dir)
        make_dmg(branch_dmg_name, dmg_dir, output_dir)


def make_parser():
    parser = argparse.ArgumentParser()
    parser.add_argument("--root-dir", default=".")
    parser.add_argument("--unix-path", default="unix/Cargo.toml")
    parser.add_argument("--glutin-path", default="glutin/Cargo.toml")
    parser.add_argument("--package-path", default="package")
    parser.add_argument("--release", action="store_true", default=False)
    parser.add_argument("--target-dir", default="target")
    parser.add_argument("--output-dir", default="uploads")
    parser.add_argument("--name", required=True)
    parser.add_argument("--os", required=True, choices=["linux", "macos"])
    parser.add_argument("--macos-app-name", required=False, type=str)
    parser.add_argument(
        "--branch", default=sh.git("rev-parse", "--abbrev-ref", "HEAD").strip()
    )
    return parser


def main(args):
    build(
        args.name,
        args.os,
        args.root_dir,
        args.unix_path,
        args.glutin_path,
        args.release,
        args.branch,
        args.target_dir,
        args.output_dir,
        args.macos_app_name,
        args.package_path,
    )


if __name__ == "__main__":
    main(make_parser().parse_args())
