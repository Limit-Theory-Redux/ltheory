cmake_minimum_required (VERSION 3.0.2)
project (LibPHX)
include (script/build/Shared.cmake)

# ------------------------------------------------------------------------------

file (GLOB HEADERS "include/*.h")
file (GLOB SOURCES "src/*.cpp")

add_library (phx SHARED ${SOURCES} ${HEADERS})
phx_configure_output_dir (phx)
phx_configure_target_properties (phx)

target_compile_definitions (phx PRIVATE LIBPHX_BUILDING=1)
target_include_directories (phx PUBLIC "include")

set_target_properties (phx PROPERTIES
  PREFIX "lib"
  OUTPUT_NAME_DEBUG "phx${ARCH}d"
  OUTPUT_NAME_RELEASE "phx${ARCH}"
  OUTPUT_NAME_RELWITHDEBINFO "phx${ARCH}"
  OUTPUT_NAME_MINSIZEREL "phx${ARCH}")

# ------------------------------------------------------------------------------

include (script/build/External.cmake)

target_link_libraries (phx PUBLIC
  BulletCollision
  BulletDynamics
  LinearMath
  fmod
  freetype
  glew_s
  luajit
  lz4_static
  SDL2-static
  SDL2main
  stb)

if (UNIX)
  target_link_libraries(phx PUBLIC dl)
endif ()

if (APPLE)
  find_library(OPENGL_LIBRARY OpenGL)
  target_link_libraries(phx PUBLIC ${OPENGL_LIBRARY})
endif ()

# Some libraries are distributed as shared libraries. These need to be copied to the bin directory at build time.
function (copy_library_to_bin target lib)
  add_custom_command (TARGET ${target} POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
    $<TARGET_FILE:${lib}> $<TARGET_FILE_DIR:${target}>
    )
endfunction ()

copy_library_to_bin(phx fmod)
