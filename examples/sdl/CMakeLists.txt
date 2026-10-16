cmake_minimum_required(VERSION 3.20)

project(sdl_example)

find_package(SDL3 REQUIRED CONFIG COMPONENTS SDL3)
find_package(ACCESSKIT REQUIRED)

add_executable(hello_world hello_world.c)
target_link_libraries(hello_world PUBLIC SDL3::SDL3)
target_link_libraries(hello_world PUBLIC accesskit)
target_compile_definitions(hello_world PRIVATE -DUNICODE -D_UNICODE)
