#!/usr/bin/env python3

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
