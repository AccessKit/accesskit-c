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

# Simple script to check whether rustc-version is to be checkedn against minimum supported rust version

import sys
from argparse import ArgumentParser

if __name__ == "__main__":
    parser = ArgumentParser()
    parser.add_argument('--string', help='String to check is a version-like string', required=True)
    args = parser.parse_args()
    parts = args.string.split('.')
    if len(parts) != 2 and len(parts) != 3:
        print('skip')
    else:
        for p in parts:
            try:
                int(p)
            except ValueError:
                print('skip')
                sys.exit()
        print('check')
