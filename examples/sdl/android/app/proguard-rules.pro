# Add project specific ProGuard rules here.

# Keep native method names
-keepclasseswithmembernames class * {
    native <methods>;
}

-keep class dev.accesskit.sdl_example.AccessKitSDLActivity { *; }
