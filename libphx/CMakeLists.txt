cmake_minimum_required (VERSION 3.0.2)
project (LibPHX)
include (script/build/Shared.cmake)

set(CMAKE_BUILD_WITH_INSTALL_RPATH TRUE)
set(CMAKE_INSTALL_RPATH "\$ORIGIN")

# ------------------------------------------------------------------------------

file (GLOB HEADERS "include/*.h")
file (GLOB SOURCES "src/*.cpp")

add_library (phx SHARED ${SOURCES} ${HEADERS})
phx_configure_output_dir (phx)
phx_configure_target_properties (phx)

# Grab dependencies
include (FetchContent)
FetchContent_Declare (
  ext
  URL https://github.com/Limit-Theory-Redux/ltheory/releases/download/v0.0.1-pre/ext.zip
)
message ("Ensuring that dependencies are up to date...")
FetchContent_MakeAvailable (ext)

target_compile_definitions (phx PRIVATE LIBPHX_BUILDING=1)
target_include_directories (phx PUBLIC "include")
target_include_directories (phx PUBLIC "${ext_SOURCE_DIR}/include")
target_include_directories (phx PUBLIC "${ext_SOURCE_DIR}/include/bullet")
target_link_directories (phx PUBLIC "${ext_SOURCE_DIR}/lib/${PLATARCH}")

set_target_properties (phx PROPERTIES
  PREFIX "lib"
  OUTPUT_NAME_DEBUG "phx${ARCH}d"
  OUTPUT_NAME_RELEASE "phx${ARCH}"
  OUTPUT_NAME_RELWITHDEBINFO "phx${ARCH}"
  OUTPUT_NAME_MINSIZEREL "phx${ARCH}")

# ------------------------------------------------------------------------------

if (WINDOWS)

  # register an external shared library
  function (add_extlib name lib dll)
    add_library (${name} SHARED IMPORTED GLOBAL)
    set_property (TARGET ${name} PROPERTY IMPORTED_IMPLIB "${ext_SOURCE_DIR}/lib/${PLATARCH}/${lib}")
    set_property (TARGET ${name} PROPERTY IMPORTED_LOCATION "${ext_SOURCE_DIR}/lib/${PLATARCH}/${dll}")
  endfunction ()

  # link target to an external library; also add a post-build step to
  # copy the library DLL into the bin folder
  function (target_link_extlib target lib)
    target_link_libraries (${target} ${lib})
    add_custom_command (TARGET ${target} POST_BUILD
      COMMAND ${CMAKE_COMMAND} -E copy_if_different
      $<TARGET_FILE:${lib}> $<TARGET_FILE_DIR:${target}>
    )
  endfunction ()

  add_extlib (glew glew32.lib glew32.dll)
  add_extlib (lz4 liblz4.lib liblz4.dll)
  add_extlib (lua51 lua51.lib lua51.dll)
  add_extlib (sdl2 SDL2.lib SDL2.dll)

  if (ARCH EQUAL "32")
    add_extlib (fmod fmodL_vc.lib fmodL.dll)
    add_extlib (fmodstudio fmodstudioL_vc.lib fmodstudioL.dll)
  else ()
    add_extlib (fmod fmodL64_vc.lib fmodL64.dll)
    add_extlib (fmodstudio fmodstudioL64_vc.lib fmodstudioL64.dll)
  endif ()

elseif (LINUX)

  # register an external shared library
  function (add_extlib name lib)
    add_library (${name} SHARED IMPORTED GLOBAL)
    set_property (TARGET ${name} PROPERTY IMPORTED_LOCATION "${ext_SOURCE_DIR}/lib/${PLATARCH}/${lib}")
  endfunction ()

  # link target to an external library; also add a post-build step to
  # copy the library shared object into the bin folder
  function (target_link_extlib target lib)
    target_link_libraries (${target} ${lib})
    add_custom_command (TARGET ${target} POST_BUILD
      COMMAND ${CMAKE_COMMAND} -E copy_if_different
      $<TARGET_FILE:${lib}> $<TARGET_FILE_DIR:${target}>
    )
  endfunction ()

  add_extlib (fmod libfmodL.so)
  add_extlib (fmodstudio libfmodstudioL.so)

  # As we're using system installed libraries, strip away the includes we don't need.
  # This is a hack until we use proper dependency management in the future.
  if (NOT ${ext_SOURCE_DIR} STREQUAL "")
    execute_process(COMMAND rm -rf
      ${ext_SOURCE_DIR}/include/bullet
      ${ext_SOURCE_DIR}/include/freetype
      ${ext_SOURCE_DIR}/include/lz4
      ${ext_SOURCE_DIR}/include/sdl)
  endif ()

  find_package(Bullet REQUIRED)
  find_package(Freetype REQUIRED)
  target_include_directories (phx PUBLIC "${BULLET_INCLUDE_DIR}")
  target_include_directories (phx PUBLIC "${FREETYPE_INCLUDE_DIRS}")

endif ()

# ------------------------------------------------------------------------------

if (WINDOWS)

  target_link_libraries (phx
    opengl32.lib
    user32.lib winmm.lib Ws2_32.lib
    freetype.lib
    BulletCollision.lib BulletDynamics.lib LinearMath.lib)

  target_link_extlib (phx glew)
  target_link_extlib (phx sdl2)
  target_link_extlib (phx lz4)
  target_link_extlib (phx lua51)
  target_link_extlib (phx fmod)
  target_link_extlib (phx fmodstudio)

elseif (LINUX)

  target_link_libraries (phx
    GL
    GLEW
    dl
    SDL2
    freetype
    luajit-5.1
    lz4
    BulletCollision BulletDynamics)

  target_link_extlib (phx fmod)
  target_link_extlib (phx fmodstudio)

  # For some reason, linking libfmodL.so causes the linker to require libfmodL.so.10, so make it happy using
  # a symlink.
  add_custom_command (TARGET phx POST_BUILD COMMAND ln -sf libfmodL.so $<TARGET_FILE_DIR:phx>/libfmodL.so.10)

endif ()
