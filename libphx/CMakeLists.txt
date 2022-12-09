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

target_compile_definitions (phx PRIVATE LIBPHX_BUILDING=1)
target_include_directories (phx PUBLIC "include")

set_target_properties (phx PROPERTIES
  PREFIX "lib"
  OUTPUT_NAME_DEBUG "phx${ARCH}d"
  OUTPUT_NAME_RELEASE "phx${ARCH}"
  OUTPUT_NAME_RELWITHDEBINFO "phx${ARCH}"
  OUTPUT_NAME_MINSIZEREL "phx${ARCH}")

# ------------------------------------------------------------------------------

if(NOT EXISTS "${CMAKE_BINARY_DIR}/conan.cmake")
  message(STATUS "Downloading conan.cmake from https://github.com/conan-io/cmake-conan")
  file(DOWNLOAD "https://raw.githubusercontent.com/conan-io/cmake-conan/0.18.1/conan.cmake"
    "${CMAKE_BINARY_DIR}/conan.cmake"
    TLS_VERIFY ON)
endif()

include(${CMAKE_BINARY_DIR}/conan.cmake)

list(APPEND CMAKE_MODULE_PATH ${CMAKE_CURRENT_BINARY_DIR})
list(APPEND CMAKE_PREFIX_PATH ${CMAKE_CURRENT_BINARY_DIR})

conan_cmake_configure(
  REQUIRES
    bullet3/3.24
    freetype/2.12.1
    glew/2.2.0
    lz4/1.9.4
    sdl/2.26.0
  GENERATORS
    cmake_find_package)
conan_cmake_autodetect(settings)
conan_cmake_install(PATH_OR_REFERENCE .
  BUILD missing
  REMOTE conancenter
  SETTINGS ${settings})

find_package(Bullet)
find_package(Freetype)
find_package(GLEW)
find_package(lz4)
find_package(SDL2)

include (FetchContent)

# stb
FetchContent_Declare (
  stb
  URL https://github.com/nothings/stb/archive/8b5f1f37b5b75829fc72d38e7b5d4bcbf8a26d55.tar.gz
  URL_MD5 5e0a1100edc552cd25d392ab615f8eec
)
FetchContent_GetProperties (stb)
if (NOT stb_POPULATED)
  FetchContent_Populate (stb)
  add_library(stb INTERFACE)
  target_include_directories(stb INTERFACE ${stb_SOURCE_DIR})
endif()

# FMOD
FetchContent_Declare (
  fmod
  URL https://github.com/Limit-Theory-Redux/ltheory/releases/download/v0.0.1-pre/fmod-2.02.08.zip
  URL_MD5 52271e15ae1dfad9a7eafb2a422d9745
)
message (STATUS "Downloading FMOD")
FetchContent_MakeAvailable (fmod)

add_library (FMOD::FMOD SHARED IMPORTED)
target_include_directories (FMOD::FMOD INTERFACE "${fmod_SOURCE_DIR}/include")
if (WIN32)
  set_property (TARGET FMOD::FMOD PROPERTY IMPORTED_LOCATION
    "${fmod_SOURCE_DIR}/lib/win/x86$<$<STREQUAL:${ARCH},64>:_64>/fmod.dll")
  set_property (TARGET FMOD::FMOD PROPERTY IMPORTED_IMPLIB
    "${fmod_SOURCE_DIR}/lib/win/x86$<$<STREQUAL:${ARCH},64>:_64>/fmod_vc.lib")
elseif (APPLE)
  set_property (TARGET FMOD::FMOD PROPERTY IMPORTED_LOCATION
    "${fmod_SOURCE_DIR}/lib/macos/libfmod.dylib")
else ()
  if (ARCH_X86)
    set_property (TARGET FMOD::FMOD PROPERTY IMPORTED_LOCATION
      "${fmod_SOURCE_DIR}/lib/linux/x86$<$<STREQUAL:${ARCH},64>:_64>/libfmod.so")
  else ()
    set_property (TARGET FMOD::FMOD PROPERTY IMPORTED_LOCATION
      "${fmod_SOURCE_DIR}/lib/linux/arm<$<STREQUAL:${ARCH},64>:64>/libfmod.so")
  endif ()
endif ()

# Luajit
FetchContent_Declare (
  luajit
  URL https://github.com/LuaJIT/LuaJIT/archive/de2e1ca9d3d87e74c0c20c1e4ad3c32b31a5875b.tar.gz
  URL_MD5 df7b3ee3b03411249639b38ae0b39c88
)
message (STATUS "Downloading luajit")
FetchContent_MakeAvailable (luajit)

file(GLOB_RECURSE LUAJIT_SRCS ${luajit_SOURCE_DIR}/src/*.c)
file(GLOB_RECURSE LUAJIT_HDRS ${luajit_SOURCE_DIR}/src/*.h(pp)?)
if (WIN32)
  add_custom_command(
    COMMAND msvcbuild.bat static
    COMMAND ${CMAKE_COMMAND} -E copy
      ${luajit_SOURCE_DIR}/src/lua51.lib
      ${luajit_BINARY_DIR}/lib/luajit-5.1.lib
    WORKING_DIRECTORY ${luajit_SOURCE_DIR}/src
    OUTPUT ${luajit_BINARY_DIR}/lib/${CMAKE_STATIC_LIBRARY_PREFIX}luajit-5.1${CMAKE_STATIC_LIBRARY_SUFFIX}
    DEPENDS ${LUAJIT_SRCS} ${LUAJIT_HDRS}
  )
elseif (APPLE)
  # Extract the SDK version from the macOS SDK sysroot. The SDK name will be of the form MacOSX00.0.sdk.
  get_filename_component(osx_sdk_name ${CMAKE_OSX_SYSROOT} NAME)
  string(SUBSTRING ${osx_sdk_name} 6 -1 osx_sdk_version_with_prefix)
  string(LENGTH ${osx_sdk_version_with_prefix} osx_sdk_substr_len)
  MATH(EXPR osx_sdk_substr_len "${osx_sdk_substr_len}-4")
  string(SUBSTRING ${osx_sdk_version_with_prefix} 0 ${osx_sdk_substr_len} osx_sdk_version)

  add_custom_command(
    COMMAND make MACOSX_DEPLOYMENT_TARGET=${osx_sdk_version} amalg
    COMMAND make PREFIX=${luajit_BINARY_DIR} install
    WORKING_DIRECTORY ${luajit_SOURCE_DIR}
    OUTPUT ${luajit_BINARY_DIR}/lib/${CMAKE_STATIC_LIBRARY_PREFIX}luajit-5.1${CMAKE_STATIC_LIBRARY_SUFFIX}
    DEPENDS ${LUAJIT_SRCS} ${LUAJIT_HDRS}
  )
else ()
  add_custom_command(
    COMMAND make MACOSX_DEPLOYMENT_TARGET=${osx_sdk_version} amalg
    COMMAND make PREFIX=${luajit_BINARY_DIR} install
    WORKING_DIRECTORY ${luajit_SOURCE_DIR}
    OUTPUT ${luajit_BINARY_DIR}/lib/${CMAKE_STATIC_LIBRARY_PREFIX}luajit-5.1${CMAKE_STATIC_LIBRARY_SUFFIX}
    DEPENDS ${LUAJIT_SRCS} ${LUAJIT_HDRS}
  )
endif ()

add_library (luajit INTERFACE
  "${luajit_BINARY_DIR}/lib/${CMAKE_STATIC_LIBRARY_PREFIX}luajit-5.1${CMAKE_STATIC_LIBRARY_SUFFIX}")
file (MAKE_DIRECTORY "${luajit_BINARY_DIR}/include")
target_include_directories (luajit INTERFACE "${luajit_BINARY_DIR}/include")
set_property (TARGET luajit PROPERTY INTERFACE_LINK_LIBRARIES
  "${luajit_BINARY_DIR}/lib/${CMAKE_STATIC_LIBRARY_PREFIX}luajit-5.1${CMAKE_STATIC_LIBRARY_SUFFIX}")

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

elseif (MACOS)

#  # register an external shared library
#  function (add_extlib name lib)
#    add_library (${name} SHARED IMPORTED GLOBAL)
#    set_property (TARGET ${name} PROPERTY IMPORTED_LOCATION "${CMAKE_CURRENT_SOURCE_DIR}/ext/lib/${PLATARCH}/${lib}.dylib")
#  endfunction ()
#
#  # link target to an external library; also add a post-build step to
#  # copy the library DLL into the bin folder
#  function (target_link_extlib target lib)
#    target_link_libraries (${target} ${lib})
#    add_custom_command (TARGET ${target} POST_BUILD
#            COMMAND ${CMAKE_COMMAND} -E copy_if_different
#            $<TARGET_FILE:${lib}> $<TARGET_FILE_DIR:${target}>
#            )
#  endfunction ()
#
#  add_extlib (glew glew32.lib glew32.dll)
#  add_extlib (lz4 liblz4.lib liblz4.dll)
#  add_extlib (lua51 lua51.lib lua51.dll)
#  add_extlib (sdl2 SDL2.lib SDL2.dll)
#
#  if (ARCH EQUAL "32")
#    add_extlib (fmod fmodL_vc.lib fmodL.dll)
#    add_extlib (fmodstudio fmodstudioL_vc.lib fmodstudioL.dll)
#  else ()
#    add_extlib (fmod fmodL64_vc.lib fmodL64.dll)
#    add_extlib (fmodstudio fmodstudioL64_vc.lib fmodstudioL64.dll)
#  endif ()

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

elseif (MACOS)

  find_library(OPENGL_LIBRARY OpenGL)
  target_link_libraries (phx PUBLIC
    dl
    ${OPENGL_LIBRARY}
    stb
    luajit
    Bullet::Bullet
    Freetype::Freetype
    GLEW::GLEW
    lz4::lz4
    SDL2::SDL2
    FMOD::FMOD)

endif ()
