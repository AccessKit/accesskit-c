cmake_minimum_required(VERSION 3.22.1)
project(hello_world)
set(JNILIBS_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../jniLibs/${ANDROID_ABI}")
set(SDL2_DIR "" CACHE PATH "Path to SDL2 source/headers for Android")

if(SDL2_DIR STREQUAL "")
    message(FATAL_ERROR "SDL2_DIR must be set to the SDL2 source directory (containing include/SDL.h)")
endif()
add_library(SDL2 SHARED IMPORTED)
set_target_properties(SDL2 PROPERTIES
    IMPORTED_LOCATION "${JNILIBS_DIR}/libSDL2.so"
)
set(SDL2_INCLUDE_DIR "${SDL2_DIR}/include")
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
    ${SDL2_INCLUDE_DIR}
    ${ACCESSKIT_INCLUDE_DIR}
)

target_link_libraries(hello_world
    SDL2
    accesskit
    android
    log
)

