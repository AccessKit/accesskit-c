cmake_minimum_required(VERSION 3.20)

project(sdl_example)

find_package(SDL3 REQUIRED CONFIG COMPONENTS SDL3)
find_package(ACCESSKIT REQUIRED)

add_executable(hello_world hello_world.c)
target_link_libraries(hello_world PUBLIC SDL3::SDL3)
target_link_libraries(hello_world PUBLIC accesskit)
target_compile_definitions(hello_world PRIVATE -DUNICODE -D_UNICODE)

if (CMAKE_SYSTEM_NAME STREQUAL "iOS")
    # The Xcode generator can only embed imported frameworks by path.
    get_target_property(_sdl3_location SDL3::SDL3-shared IMPORTED_LOCATION)
    get_filename_component(_sdl3_framework "${_sdl3_location}" DIRECTORY)
    set_target_properties(hello_world PROPERTIES
        MACOSX_BUNDLE TRUE
        MACOSX_BUNDLE_INFO_PLIST "${CMAKE_CURRENT_SOURCE_DIR}/ios/Info.plist.in"
        MACOSX_BUNDLE_BUNDLE_NAME "Hello world"
        MACOSX_BUNDLE_GUI_IDENTIFIER "dev.accesskit.sdl-example"
        MACOSX_BUNDLE_BUNDLE_VERSION "1"
        MACOSX_BUNDLE_SHORT_VERSION_STRING "1.0"
        XCODE_ATTRIBUTE_PRODUCT_BUNDLE_IDENTIFIER "dev.accesskit.sdl-example"
        XCODE_ATTRIBUTE_IPHONEOS_DEPLOYMENT_TARGET "18.0"
        XCODE_EMBED_FRAMEWORKS "${_sdl3_framework}"
        XCODE_EMBED_FRAMEWORKS_CODE_SIGN_ON_COPY ON
        XCODE_ATTRIBUTE_LD_RUNPATH_SEARCH_PATHS "@executable_path/Frameworks"
    )
endif()
