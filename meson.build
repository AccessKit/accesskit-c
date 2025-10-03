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

project('accesskit-c',
        'c',
        meson_version: '>= 1.3.0',
        default_options: [
          'pkgconfig.relocatable=true',
        ]
)

host_system = host_machine.system()
cc = meson.get_compiler('c')

# MSRV - Minimum Supported Rust Version
# If you change this, please update these:
#   - the "rust-version" value in Cargo.toml
msrv = '1.77.2'
cargo = find_program('cargo', version:'>= @0@'.format(msrv))
cargo_wrapper = find_program('meson/cargo_wrapper.py', native: true)
rustc = find_program('rustc', version:'>= @0@'.format(msrv))
makedef = find_program('meson/makedef.py', native: true)
project_version_script = find_program('meson/project_version.py', native: true)

py = import('python')

python = py.find_installation()

appkit_dep = dependency('appleframeworks', modules: 'AppKit',
                        required: host_system == 'darwin')
m_dep = cc.find_library('m', required: false)
uiautomationcore_dep = cc.find_library('uiautomationcore',
                                       required: host_system == 'windows')
runtimeobject_dep = cc.find_library('runtimeobject',
                                    required: host_system == 'windows')
propsys_dep = cc.find_library('propsys',
                              required: host_system == 'windows')

library_dependencies_sole = []
library_dependencies = library_dependencies_sole
private_dependencies = []
other_library_dependencies = []

target = get_option('triplet')
target_arg = []

toolchain_arg = []
toolchain_ver = get_option('rustc-version')

check_version = false
# Check toolchain_ver if supplied, if in the form of a version
# We assume to continue if a stable, nightly or beta or custom toolchain is specified
if toolchain_ver != ''
  if host_system != 'windows'
    warning('\'rustc-version\' option currently only supported for Windows')
  endif
  check_version_script = find_program('meson/check-is-version-string.py', native: true)
  is_version_string_check = run_command(
    [check_version_script, '--string', toolchain_ver],
    capture: true,
    check: true
  )
  check_version = is_version_string_check.stdout().split()[0] == 'check'
  if check_version
    if not toolchain_ver.version_compare('>=@0@'.format(msrv))
      error('Specified Rust toolchain version @0@ is less than @1@'.format(toolchain_ver, msrv))
    endif
  endif
endif

# Ideally, cc.get_argument_syntax() == 'msvc' should
# cover this, but 'clang' can mean GCC or MSVC-style
# depending on the env and/or flags on Windows
is_msvc_style = cc.get_argument_syntax() == 'msvc' or cc.get_define('_MSC_VER') != ''

if target != ''
  target_arg += ['--target', target]
endif

if host_system == 'windows'
  query_rustc_harness = find_program('meson/query-rustc.py', native: true, required: get_option('default_library') != 'shared')

  rustc_query_args = [
    query_rustc_harness,
    rustc,
  ]

  # First deduce the target that is to be used for the build, based on Meson build settings
  host_cpu_family = host_machine.cpu_family()
  rustc_toolchain_cpu = host_cpu_family == 'x86' ? 'i686' : host_cpu_family
  if is_msvc_style
    rustc_toolchain_cc = 'msvc'
  elif cc.get_id() == 'clang'
    rustc_toolchain_cc = 'gnullvm'
  else
    rustc_toolchain_cc = 'gnu'
  endif
  rustc_target = '@0@-pc-windows-@1@'.format(rustc_toolchain_cpu, rustc_toolchain_cc)

  if toolchain_ver != ''
    build_triplet_arg = []
    if target_arg == []
      target_arg += ['--target', rustc_target]
    endif

    # If doing a cross build, with '-Drustc-version=...' we must find the native toolchain on the build
    # machine with its arch triplet that is to be used for building against the target, as Cargo will
    # require this in its command line
    if meson.is_cross_build()
      build_cpu_family = build_machine.cpu_family()
      rustc_build_toolchain_cpu = build_cpu_family == 'x86' ? 'i686' : build_cpu_family
      rustc_build_triplet = '@0@-pc-windows-@1@'.format(rustc_build_toolchain_cpu, rustc_toolchain_cc)
      build_triplet_arg += ['--build-triplet', rustc_build_triplet]
    endif

    # If a version number is used, also check against the actual release that is depicted
    # by the currently installed 'stable' toolchain, and use the stable toolchain if the requested
    # version matches the installed 'stable' toolchain
    if check_version
      stable_toolchain_arg = ['--toolchain-version', 'stable']

      stable_actual_version = run_command(
        rustc_query_args + ['--query=stable-actual-version'] + stable_toolchain_arg + target_arg + build_triplet_arg,
        capture: true,
        check: true
      )
      if stable_actual_version.stderr() != ''
        error('error occurred when querying stable toolchain: @0@'.format(default_host.stderr().split()[0]))
      endif
      stable_version = stable_actual_version.stdout().split()[0]
      if stable_version == toolchain_ver
        toolchain_arg += stable_toolchain_arg
      endif
    endif
    if toolchain_arg == []
      toolchain_arg += ['--toolchain-version', toolchain_ver]
    endif
    toolchain_arg += toolchain_arg + build_triplet_arg
  endif

  if target_arg == []
    default_host = run_command(
      rustc_query_args + ['--query=default-host-toolchain'] + toolchain_arg,
      capture: true,
      check: true
    )
    if default_host.stderr() != ''
      error('error occurred when querying default toolchain: @0@'.format(default_host.stderr().split()[0]))
    endif
    default_rustc_toolchain = default_host.stdout().split()[0]

    # If the default Rust target triplet does not match the triplet that we want to build for
    # pass in a --target argument to RustC/Cargo that best matches what we are building for,
    # if we didn't use -Dtriplet=... in the Meson setup line.  Do the same if a toolchain version
    # is requested
    if default_rustc_toolchain != rustc_target
      target_arg += ['--target', rustc_target]
    endif
  endif

  rustc_query_native_static_libs_args = rustc_query_args
  if target_arg != []
    rustc_query_native_static_libs_args += target_arg
  endif
  if toolchain_arg != []
    rustc_query_native_static_libs_args += toolchain_arg
  endif
  rustc_query_native_static_libs_args += ['--query=native-static-libs']

  native_libs = run_command(
    rustc_query_native_static_libs_args,
    capture: true,
    check: true
  )

  if native_libs.stderr() != ''
    error(native_libs.stderr())
  endif

  foreach i: native_libs.stdout().split()
    if i != 'msvcrt'
      private_dependencies += cc.find_library(i, required: get_option('default_library') != 'shared')
    endif
  endforeach
endif

private_dependencies += [m_dep, appkit_dep, uiautomationcore_dep, runtimeobject_dep, propsys_dep]
library_dependencies += other_library_dependencies
library_dependencies += private_dependencies

cargo_toml = meson.project_source_root() / 'Cargo.toml'

project_version_args = [
  '--cargo', cargo.full_path(),
  '--manifest-path', cargo_toml,
]
project_version_result = run_command(
  [project_version_script, project_version_args],
  capture: true,
  check: true
)
project_version = project_version_result.stdout().strip()
version_parts = project_version.split('.')
if version_parts[0] == '0'
  api_version = '@0@.@1@'.format(version_parts[0], version_parts[1])
else
  api_version = version_parts[0]
endif

accesskit_c_lib_name = 'accesskit-c-@0@'.format(api_version)

includeinc = include_directories('include')

# Set the suffixes up
if host_system == 'windows'
  lib_prefix = is_msvc_style ? '' : 'lib'
  ext_dynamic = 'dll'
  ext_static = is_msvc_style ? 'lib' : 'a'
  ext_exe = '.exe'
elif host_system in ['darwin', 'ios']
  lib_prefix = 'lib'
  ext_dynamic = 'dylib'
  ext_static = 'a'
  ext_exe = ''
else
  lib_prefix = 'lib'
  ext_dynamic = 'so'
  ext_static = 'a'
  ext_exe = ''
endif

cargo_wrapper_args = [
  '--cargo', cargo.full_path(),
  '--manifest-path', cargo_toml,
  '--project-build-root', meson.project_build_root(),
]

if not get_option('debug') or \
   (is_msvc_style and get_option('buildtype') == 'debugoptimized')
  cargo_wrapper_args += ['--release']
endif

if target_arg != []
  cargo_wrapper_args += target_arg
endif
if toolchain_arg != []
  cargo_wrapper_args += toolchain_arg
endif

# Header generation with cbindgen
generated_headers_directory = meson.project_source_root() / 'include'
rustup = find_program('rustup', required: true)
cbindgen = find_program('cbindgen', required: false)
clang_format = find_program('clang-format', required: false)

if cbindgen.found() and clang_format.found()
  # Generate header with cbindgen, then format
  generated_header = custom_target(
    'accesskit_unformatted.h',
    output: 'accesskit_unformatted.h',
    command: [
      rustup, 'run', 'nightly-2025-03-27',
      cbindgen, '--crate', 'accesskit-c', '--output', '@OUTPUT@',
      meson.project_source_root()
    ],
    build_by_default: false,
  )

  formatted_header = custom_target(
    'accesskit.h',
    input: generated_header,
    output: 'accesskit.h',
    command: [clang_format, '@INPUT@'],
    capture: true,
    build_by_default: false,
    install: true,
    install_dir: generated_headers_directory,
  )
endif

includes = files(
    generated_headers_directory / 'accesskit.h'
)

install_headers(
    includes,
    subdir: '@0@'.format(accesskit_c_lib_name),
)

library_sources = files(
  'Cargo.lock',
  'Cargo.toml',
  'src/common.rs',
  'src/geometry.rs',
  'src/lib.rs',
  'src/macos.rs',
  'src/unix.rs',
  'src/windows.rs',
)

rust_artifacts = custom_target(
  'accesskit-c',
  build_by_default: true,
  output: '@0@accesskit.@1@'.format(lib_prefix, ext_static),
  console: true,
  depend_files: library_sources,
  command: [
    cargo_wrapper,
    cargo_wrapper_args,
    '--command=cbuild',
    '--current-build-dir', '@OUTDIR@',
    '--current-source-dir', meson.current_source_dir(),
    '--extension', ext_static,
  ],
)

accesskit_c_lib_internal = rust_artifacts[0]

makedef_args = [
  makedef,
  '--regex',
  '^accesskit_.',
]

if host_system in ['darwin', 'ios']
  makedef_args += ['--os', 'darwin']
elif host_system in ['windows', 'cygwin']
  makedef_args += ['--os', 'win']
else
  makedef_args += ['--os', 'linux']
endif

if cc.symbols_have_underscore_prefix()
  makedef_args += ['--prefix', '_']
else
  makedef_args += ['--prefix', '']
endif

makedef_tool_args = []

if is_msvc_style
  dumpbin = find_program('dumpbin')
  makedef_tool_args += ['--dumpbin', dumpbin]
else
  nm = find_program('llvm-nm', required: false)
  if not nm.found()
    if host_system in ['darwin', 'ios']
      warning('llvm-nm not found, you may experience problems when creating the shared libaccesskit-c.')
      warning('Please install the llvm-tools component through rustup, or point Meson to an existing LLVM installation.')
    endif
    nm = find_program('nm')
  endif
  makedef_tool_args += ['--nm', nm]
endif

symbol_list_target = custom_target(
  'accesskit-c.def',
  command: [makedef_args, makedef_tool_args, '@INPUT@'],
  input: accesskit_c_lib_internal,
  output: 'accesskit-c.def',
  capture: true,
)

symbol_list = symbol_list_target[0]

if host_system in ['darwin', 'ios']
  vflags = ['-Wl,-exported_symbols_list,@0@'.format(symbol_list_target.full_path())]
elif host_system == 'windows'
  vflags = []
else
  vflags = ['-Wl,--version-script,@0@'.format(symbol_list_target.full_path())]
endif

# This is not strictly needed, but since we are telling Cargo to build a staticlib, it puts in
# all of Rust's standard library and code from dependencies even when it is not needed.
# With the following, we shrink the final .so size.
#
# Also check for -Bsymbolic-functions linker flag used to avoid
# intra-library PLT jumps, if available.
strip_link_args = cc.get_supported_link_arguments(
  '-Wl,-dead_strip',
  '-Wl,--gc-sections',
  '-Wl,-Bsymbolic-functions',
)

link_args = vflags + strip_link_args

# Some systems, reportedly OpenBSD and macOS, refuse
# to create libraries without any object files. Compiling
# this file, and adding its object file to the library,
# will prevent the library from being empty.
if cc.has_function_attribute('unused')
  accesskit_c_dummy = configure_file(
    command: [
      python,
      '-c',
      'print("static int __attribute__((unused)) __accesskit_c_dummy;")'
    ],
    capture: true,
    output: '_accesskit_c_dummy.c'
  )
else
  accesskit_c_dummy = configure_file(
    command: [
      python,
      '-c',
      'print("static int __accesskit_c_dummy; int dummy(void) { return __accesskit_c_dummy; }")'
    ],
    capture: true,
    output: '_accesskit_c_dummy.c'
  )
endif

if get_option('default_library') in ['shared', 'both']
  if host_system == 'windows'
    accesskit_c_shared_lib = shared_library(
      accesskit_c_lib_name,
      accesskit_c_dummy,
      link_with: accesskit_c_lib_internal,
      link_args: link_args,
      link_depends: symbol_list,
      dependencies: library_dependencies,
      include_directories: [includeinc],
      vs_module_defs: symbol_list,
      install: true,
    )
  else
    accesskit_c_shared_lib = shared_library(
      accesskit_c_lib_name,
      accesskit_c_dummy,
      link_whole: accesskit_c_lib_internal,
      link_args: link_args,
      link_depends: symbol_list,
      dependencies: library_dependencies,
      include_directories: [includeinc],
      vs_module_defs: symbol_list,
      install: true,
      version: project_version,
    )
  endif
  if get_option('default_library') == 'shared'
    accesskit_c_lib = accesskit_c_shared_lib
  endif
endif
if get_option('default_library') in ['static', 'both']
  # The static library must be manually generated so that it matches Meson's
  # naming convention.
  accesskit_c_lib = custom_target(
    'accesskit-c-static',
    input: accesskit_c_lib_internal,
    output: '@0@@1@.@2@'.format(lib_prefix, accesskit_c_lib_name, 'a'),
    command: [
      python,
      '-c',
      'import os; import sys; import shutil; shutil.copy(sys.argv[1], sys.argv[2])',
      '@INPUT0@',
      '@OUTPUT0@'
    ],
    install: true,
    install_dir: get_option('libdir'),
  )
endif

# This is the main dependency to use for tests; it means "link to the library we just built"
accesskit_c_dep = declare_dependency(
  sources: includes,
  include_directories: [includeinc],
  dependencies: library_dependencies,
  link_with: accesskit_c_lib,
)

meson.override_dependency(accesskit_c_lib_name, accesskit_c_dep)

pkg = import('pkgconfig')

# If any of the dependencies is e.g. wrap, ignore as we can't include
# them without knowing how they exposed the pkg-config module
# (if CMake, there's simply no way at all)
has_pkgconfig_dependencies = true

foreach i : library_dependencies_sole
  if i.found() and i.type_name() != 'pkgconfig'
    warning('One or more dependencies are not provided by pkg-config, skipping generation of the pkg-config module.')
    has_pkgconfig_dependencies = false
    break
  endif
endforeach

if has_pkgconfig_dependencies
  accesskit_c_pc = pkg.generate(
    name: 'accesskit-c',
    filebase: accesskit_c_lib_name,
    version: project_version,
    description: 'UI accessibility infrastructure (C API)',
    libraries: [accesskit_c_lib] + other_library_dependencies,
    subdirs: accesskit_c_lib_name,
    requires: library_dependencies_sole,
    libraries_private: private_dependencies,
  )
endif
