# AccessKit cross-platform SDL example

This example demonstrates how to make use of the C bindings to create cross-platform applications.

## Building

The process will vary based on your operating system.

### Windows:

First download an SDL2 development package from the project's [GitHub release page](https://github.com/libsdl-org/SDL/releases) (SDL2-devel-2.x.y-VC.zip for MSVC) and extract it.

```bash
cmake -S . -B build -DACCESSKIT_DIR="../.." -DSDL2_DIR="<PATH_TO_SDL2_PACKAGE>/cmake"
cmake --build build --config Release
```

You will then need to copy `SDL2.dll` into the `build/Release` folder.

### Linux

Make sure to install SDL2 and its development package.

```bash
cmake -S . -B build -DACCESSKIT_DIR="../.." -DCMAKE_BUILD_TYPE=Release
cmake --build build
```

### macOS

First download an SDL2 package from the project's [GitHub release page](https://github.com/libsdl-org/SDL/releases) (SDL2-2.x.y.dmg) and copy `SDL2.framework` to `/Library/Frameworks`.

```bash
cmake -S . -B build -DACCESSKIT_DIR="../.." -DCMAKE_BUILD_TYPE=Release
cmake --build build
```

### Android

You will need to build SDL2 for Android (see [SDL Android README](https://github.com/libsdl-org/SDL/blob/main/docs/README-android.md)).

#### Building the Example APK

1. Copy pre-built libraries to `android/app/src/main/jniLibs/<abi>/`
2. Configure `android/local.properties`:
   ```properties
   sdk.dir=/path/to/Android/Sdk
   sdl2.dir=/path/to/SDL2-source
   accesskit.dir=/path/to/accesskit-c
   ```

   - `sdl2.dir`: Path to SDL2 source directory containing `include/SDL.h`
   - `accesskit.dir`: Path to accesskit-c root directory containing `include/accesskit.h`

3. Build and install:
   ```bash
   cd android
   ./gradlew installDebug
   ```
