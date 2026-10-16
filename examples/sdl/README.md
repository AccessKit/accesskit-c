# AccessKit cross-platform SDL example

This example demonstrates how to make use of the C bindings to create cross-platform applications.

## Building

The process will vary based on your operating system.

### Windows:

First download an SDL3 development package from the project's [GitHub release page](https://github.com/libsdl-org/SDL/releases) (SDL3-devel-3.x.y-VC.zip for MSVC) and extract it.

```bash
cmake -S . -B build -DACCESSKIT_DIR="../.." -DSDL3_DIR="<PATH_TO_SDL3_PACKAGE>/cmake"
cmake --build build --config Release
```

You will then need to copy `SDL3.dll` into the `build/Release` folder.

### Linux

Make sure to install SDL3 and its development package.

```bash
cmake -S . -B build -DACCESSKIT_DIR="../.." -DCMAKE_BUILD_TYPE=Release
cmake --build build
```

### macOS

First download an SDL3 package from the project's [GitHub release page](https://github.com/libsdl-org/SDL/releases) (SDL3-3.x.y.dmg) and copy both `SDL3.xcframework` and `share` to `~/Library/Frameworks`.

```bash
cmake -S . -B build -DACCESSKIT_DIR="../.." -DCMAKE_BUILD_TYPE=Release
cmake --build build
```

### iOS

The SDL3 package downloaded for macOS (see above) also contains the iOS frameworks. You will also need to build accesskit-c for your iOS target: from the root of this repository, using the Rust target `aarch64-apple-ios-sim` for the simulator on Apple silicon (or `aarch64-apple-ios` with `-DCMAKE_OSX_SYSROOT=iphoneos` for a device):

```bash
cmake -S . -B build-ios -DCMAKE_BUILD_TYPE=Release -DCMAKE_SYSTEM_NAME=iOS -DCMAKE_OSX_ARCHITECTURES=arm64 -DCMAKE_OSX_SYSROOT=iphonesimulator -DRust_CARGO_TARGET=aarch64-apple-ios-sim
cmake --build build-ios
cmake --install build-ios
```

Then build the example with the Xcode generator. `SDL3_DIR` must be given explicitly, as CMake only searches the iOS SDK for packages when cross-compiling:

```bash
cmake -S . -B build-ios -G Xcode -DCMAKE_SYSTEM_NAME=iOS -DCMAKE_OSX_ARCHITECTURES=arm64 -DCMAKE_OSX_SYSROOT=iphonesimulator -DACCESSKIT_DIR="../.." -DSDL3_DIR="$HOME/Library/Frameworks/share/cmake/SDL3"
cmake --build build-ios --config Release
xcrun simctl install booted build-ios/Release-iphonesimulator/hello_world.app
xcrun simctl launch booted dev.accesskit.sdl-example
```

To run on a device, use `-DCMAKE_OSX_SYSROOT=iphoneos` and pass your signing team with `-DCMAKE_XCODE_ATTRIBUTE_DEVELOPMENT_TEAM=<TEAM_ID>`, then run the app from the generated `build-ios/sdl_example.xcodeproj`.

### Android

You will need to build SDL3 for Android (see [SDL Android README](https://github.com/libsdl-org/SDL/blob/main/docs/README-android.md)).

#### Building the Example APK

1. Copy pre-built libraries to `android/app/src/main/jniLibs/<abi>/`
2. Configure `android/local.properties`:
   ```properties
   sdk.dir=/path/to/Android/Sdk
   sdl3.dir=/path/to/SDL3-source
   accesskit.dir=/path/to/accesskit-c
   ```

   - `sdl3.dir`: Path to SDL3 source directory containing `include/SDL3/SDL.h`
   - `accesskit.dir`: Path to accesskit-c root directory containing `include/accesskit.h`

3. Build and install:
   ```bash
   cd android
   ./gradlew installDebug
   ```
