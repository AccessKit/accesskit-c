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

### OpenHarmony

Build SDL3 for OpenHarmony using the OpenHarmony NDK toolchain (see [SDL OpenHarmony README](https://github.com/libsdl-org/SDL/blob/main/docs/README-ohos.md)), and pass the OpenHarmony CMake toolchain file (`ohos.toolchain.cmake`) along with `OHOS_ARCH` (`arm64-v8a` or `x86_64`):

```bash
cmake -S . -B build \
  -DCMAKE_TOOLCHAIN_FILE="$OHOS_NDK_HOME/native/build/cmake/ohos.toolchain.cmake" \
  -DOHOS_ARCH=arm64-v8a \
  -DACCESSKIT_DIR="../.." \
  -DSDL3_DIR="<PATH_TO_SDL3_OHOS_BUILD>" \
  -DCMAKE_BUILD_TYPE=Release
cmake --build build
```

When embedding into an OpenHarmony `XComponent` host, implement `void *ohos_get_accessibility_provider(SDL_Window *window)` to return the `ArkUI_AccessibilityProvider*` obtained via `OH_NativeXComponent_GetNativeAccessibilityProvider`.
