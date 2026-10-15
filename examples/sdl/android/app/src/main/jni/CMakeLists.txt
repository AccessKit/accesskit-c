cmake_minimum_required(VERSION 3.22.1)
project(hello_world)
set(JNILIBS_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../jniLibs/${ANDROID_ABI}")
set(SDL3_DIR "" CACHE PATH "Path to SDL3 source/headers for Android")

if(SDL3_DIR STREQUAL "")
    message(FATAL_ERROR "SDL3_DIR must be set to the SDL3 source directory (containing include/SDL3/SDL.h)")
endif()
add_library(SDL3 SHARED IMPORTED)
set_target_properties(SDL3 PROPERTIES
    IMPORTED_LOCATION "${JNILIBS_DIR}/libSDL3.so"
)
set(SDL3_INCLUDE_DIR "${SDL3_DIR}/include")
set(ACCESSKIT_DIR "" CACHE PATH "Path to AccessKit C root directory")

if(ACCESSKIT_DIR STREQUAL "")
    message(FATAL_ERROR "ACCESSKIT_DIR must be set to the accesskit-c root directory (containing include/accesskit.h)")
endif()
add_library(accesskit STATIC IMPORTED)
set_target_properties(accesskit PROPERTIES
    IMPORTED_LOCATION "${JNILIBS_DIR}/libaccesskit.a"
)
set(ACCESSKIT_INCLUDE_DIR "${ACCESSKIT_DIR}/include")
set(HELLO_WORLD_SRC "${CMAKE_CURRENT_SOURCE_DIR}/../../../../../hello_world.c")
add_library(hello_world SHARED
    ${HELLO_WORLD_SRC}
    android_jni.c
)

target_include_directories(hello_world PRIVATE
    ${SDL3_INCLUDE_DIR}
    ${ACCESSKIT_INCLUDE_DIR}
)

target_link_libraries(hello_world
    SDL3
    accesskit
    android
    log
)

