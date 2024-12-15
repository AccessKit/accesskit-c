#!/usr/bin/env python3
# Derived from the librsvg Meson build
# Copyright (c) 2024 L. E. Segovia <amy@amyspark.me>
#
# This library is free software; you can redistribute it and/or
# modify it under the terms of the GNU Lesser General Public
# License as published by the Free Software Foundation; either
# version 2.1 of the License, or (at your option) any later version.
#
# This library is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
# Lesser General Public License for more details.
#
# You should have received a copy of the GNU Lesser General Public
# License along with this library; if not, see <http://www.gnu.org/licenses/>.

from argparse import ArgumentParser
import os
from pathlib import Path
import shutil
import subprocess
import sys

parser = ArgumentParser()

parser.add_argument(
    "--command",
    required=True,
    choices=["cbuild", "test", "build"],
    help="Cargo command",
)

parser.add_argument(
    "--cargo", required=True, type=Path, help="Path to the cargo executable"
)

parser.add_argument(
    "--manifest-path", required=True, type=Path, help="Path to Cargo.toml"
)

parser.add_argument(
    "--current-build-dir",
    required=True,
    type=Path,
    help="Value from meson.current_build_dir()",
)

parser.add_argument(
    "--current-source-dir",
    required=True,
    type=Path,
    help="Value from meson.current_source_dir()",
)

parser.add_argument(
    "--project-build-root",
    required=True,
    type=Path,
    help="Value from meson.project_build_root()",
)

parser.add_argument(
    "--toolchain-version", help="Rust Toolchain Version if needed"
)

parser.add_argument(
    "--target", help="Target triplet"
)

parser.add_argument(
    "--build-triplet", help="Build toolchain triplet (for cross builds using specific toolchain version)"
)

parser.add_argument(
    "--release", action="store_true", help="Build artifacts in release mode"
)

parser.add_argument(
    "--extension", required=True, help="filename extension for the library (so, a, dll, lib, dylib)",
)

args = parser.parse_args()

if args.toolchain_version is not None and args.target is None and args.build_triplet is None:
    raise ValueError('--target and/or --build-triplet argument required if --toolchain-version is specified')

if args.command == 'test':
    if args.extension or args.bin:
        raise ValueError('Cargo test does not take --extension or --bin')

cargo_target_dir = Path(args.project_build_root) / "target"

if args.target:
    cargo_target_output_dir = cargo_target_dir / args.target
else:
    cargo_target_output_dir = cargo_target_dir

env = os.environ.copy()
pkg_config_path = [i for i in env.get("PKG_CONFIG_PATH", "").split(os.pathsep) if i]
pkg_config_path.insert(
    0, (Path(args.project_build_root) / "meson-uninstalled").as_posix()
)
env["PKG_CONFIG_PATH"] = os.pathsep.join(pkg_config_path)

features = []

cargo_cmd = [Path(args.cargo).as_posix()]

if args.toolchain_version is not None:
    if args.build_triplet is not None:
        cargo_cmd.extend(["+%s-%s" % (args.toolchain_version, args.build_triplet)])
    else:
        cargo_cmd.extend(["+%s-%s" % (args.toolchain_version, args.target)])

if args.command == "cbuild":
    cargo_cmd.extend(["rustc", "--locked"])
    library_type = "staticlib" if args.extension in ("a", "lib") else "cdylib"
    cargo_cmd.extend(["--crate-type", library_type])
elif args.command == "test":
    cargo_cmd.extend(["test", "--locked", "--no-fail-fast", "--color=always"])
else:
    cargo_cmd.extend(["build", "--locked"])
    if args.bin:
        cargo_cmd.extend(["--bin", args.bin])

cargo_cmd.extend(["--manifest-path", Path(args.manifest_path).as_posix()])
cargo_cmd.extend(["--target-dir", cargo_target_dir.as_posix()])

if args.release:
    buildtype = 'release'
    cargo_cmd.extend(['--release'])
else:
    buildtype = 'debug'

if args.target:
    cargo_cmd.extend(['--target', args.target])

if features:
    cargo_cmd.extend(["--features", ",".join(features)])

if args.command == "test":
    cargo_cmd.extend(["--", "--include-ignored"])

print(f"command: {cargo_cmd}")
subprocess.run(cargo_cmd, env=env, check=True)

if args.command in ["cbuild", "build"]:
    for f in cargo_target_dir.glob(f"**/{buildtype}/*.{args.extension}"):
        shutil.copy(f, args.current_build_dir)
