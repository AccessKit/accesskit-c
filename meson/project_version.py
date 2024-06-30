#!/usr/bin/env python3

from argparse import ArgumentParser
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

parser = ArgumentParser()

parser.add_argument(
    "--cargo", required=True, type=Path, help="Path to the cargo executable"
)

parser.add_argument(
    "--manifest-path", required=True, type=Path, help="Path to Cargo.toml"
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

args = parser.parse_args()

if args.toolchain_version is not None and args.target is None and args.build_triplet is None:
    raise ValueError('--target and/or --build-triplet argument required if --toolchain-version is specified')

cargo_cmd = [Path(args.cargo).as_posix()]

if args.toolchain_version is not None:
    if args.build_triplet is not None:
        cargo_cmd.extend(["+%s-%s" % (args.toolchain_version, args.build_triplet)])
    else:
        cargo_cmd.extend(["+%s-%s" % (args.toolchain_version, args.target)])

cargo_cmd.append("read-manifest")
cargo_cmd.extend(["--manifest-path", Path(args.manifest_path).as_posix()])

result = subprocess.run(cargo_cmd, capture_output=True, check=True)
manifest = json.loads(result.stdout)
print(manifest["version"])
