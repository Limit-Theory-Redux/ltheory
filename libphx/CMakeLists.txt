cmake_minimum_required (VERSION 3.0.2)
project (LibPHX)
include (cmake/Shared.cmake)

cmake_policy(SET CMP0068 NEW)

include (cmake/External.cmake)

if(NOT TARGET libphx-external)
  add_custom_target(libphx-external)
  add_dependencies(libphx-external
    BulletCollision
    BulletDynamics
    LinearMath
#    fmod
    freetype
    glew_s
    luajit
    lz4_static)
endif()
