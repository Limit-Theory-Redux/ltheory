# ------------------------------------------------------------------------------
# Download CPM

set(CPM_DOWNLOAD_VERSION 0.36.0)

if(CPM_SOURCE_CACHE)
  set(CPM_DOWNLOAD_LOCATION "${CPM_SOURCE_CACHE}/cpm/CPM_${CPM_DOWNLOAD_VERSION}.cmake")
elseif(DEFINED ENV{CPM_SOURCE_CACHE})
  set(CPM_DOWNLOAD_LOCATION "$ENV{CPM_SOURCE_CACHE}/cpm/CPM_${CPM_DOWNLOAD_VERSION}.cmake")
else()
  set(CPM_DOWNLOAD_LOCATION "${CMAKE_BINARY_DIR}/cmake/CPM_${CPM_DOWNLOAD_VERSION}.cmake")
endif()

# Expand relative path. This is important if the provided path contains a tilde (~)
get_filename_component(CPM_DOWNLOAD_LOCATION ${CPM_DOWNLOAD_LOCATION} ABSOLUTE)
if(NOT (EXISTS ${CPM_DOWNLOAD_LOCATION}))
  message(STATUS "Downloading CPM.cmake to ${CPM_DOWNLOAD_LOCATION}")
  file(DOWNLOAD
       https://github.com/cpm-cmake/CPM.cmake/releases/download/v${CPM_DOWNLOAD_VERSION}/CPM.cmake
       ${CPM_DOWNLOAD_LOCATION}
  )
endif()

include(${CPM_DOWNLOAD_LOCATION})

# ------------------------------------------------------------------------------
# Set up the patch tool.

if(WIN32)
  CPMAddPackage(
    NAME patch
    URL https://github.com/WinMerge/patch/releases/download/v2.5.9-7/patch-2.5.9-7-bin.zip
    VERSION 2.5.9
  )
  # Windows will refuse to execute a binary called 'patch.exe', so create a copy called something else.
  execute_process(COMMAND ${CMAKE_COMMAND} -E copy ${patch_SOURCE_DIR}/bin/patch.exe ${patch_SOURCE_DIR}/bin/applydiff.exe)
  set(PATCH_TOOL ${patch_SOURCE_DIR}/bin/applydiff.exe)
else()
  find_program(PATCH_TOOL patch)
endif()

# ------------------------------------------------------------------------------
# Define dependencies

if(UNIX AND NOT APPLE)
  set(OpenGL_GL_PREFERENCE GLVND)
endif()
set(CMAKE_REQUIRED_QUIET ON)
set(X11_FIND_QUIETLY ON)

CPMAddPackage(
  NAME bullet
  URL https://github.com/bulletphysics/bullet3/archive/refs/tags/3.24.tar.gz
  VERSION 3.24
  PATCH_COMMAND ${PATCH_TOOL} -p1 -i ${CMAKE_CURRENT_SOURCE_DIR}/cmake/bullet.diff
  OPTIONS
  "BUILD_BULLET2_DEMOS OFF"
  "BUILD_BULLET3 OFF"
  "BUILD_CLSOCKET OFF"
  "BUILD_CPU_DEMOS OFF"
  "BUILD_EXTRAS OFF"
  "BUILD_ENET OFF"
  "BUILD_OPENGL3_DEMOS OFF"
  "BUILD_PYBULLET OFF"
  "BUILD_UNIT_TESTS OFF"
  "BULLET2_MULTITHREADING OFF"
  "INSTALL_CMAKE_FILES OFF"
  "INSTALL_LIBS OFF"
  "USE_GRAPHICAL_BENCHMARK OFF"
  "USE_MSVC_RUNTIME_LIBRARY_DLL ON"
  "USE_GLUT OFF"
  "USE_SOFT_BODY_MULTI_BODY_DYNAMICS_WORLD OFF"
)
if (bullet_ADDED)
  target_include_directories(BulletDynamics PUBLIC ${bullet_SOURCE_DIR}/src)
endif ()

if(MACOS)
  set(DILIGENT_NO_VULKAN ON)
else()
  set(DILIGENT_NO_VULKAN OFF)
endif()
CPMAddPackage(
  NAME DiligentCore
  URL https://github.com/DiligentGraphics/DiligentCore/releases/download/v2.5.3/DiligentCore_v2.5.3.zip
  VERSION 2.5.3
  OPTIONS
  "DILIGENT_INSTALL_CORE OFF"
  "DILIGENT_INSTALL_PDB OFF"
  "DILIGENT_NO_FORMAT_VALIDATION ON"
  "DILIGENT_NO_VULKAN ${DILIGENT_NO_VULKAN}"
  "DILIGENT_NO_METAL ON" # Metal is only supported with a pro license of Diligent.
)
if(DiligentCore_ADDED)
  target_include_directories(Diligent-Common PUBLIC ${DiligentCore_SOURCE_DIR})
endif()

CPMAddPackage(
  NAME fmod
  URL https://github.com/Limit-Theory-Redux/ltheory/releases/download/v0.0.1-pre/fmod-2.02.08.zip
  VERSION 2.02.08
  DOWNLOAD_ONLY TRUE
)
if (fmod_ADDED)
  add_library(fmod SHARED IMPORTED)
  target_include_directories(fmod INTERFACE "${fmod_SOURCE_DIR}/include")
  if (WIN32)
    set_property(TARGET fmod PROPERTY IMPORTED_LOCATION
      "${fmod_SOURCE_DIR}/lib/win/x86_64/fmod.dll")
    set_property(TARGET fmod PROPERTY IMPORTED_IMPLIB
      "${fmod_SOURCE_DIR}/lib/win/x86_64/fmod_vc.lib")
  elseif (APPLE)
    set_property(TARGET fmod PROPERTY IMPORTED_LOCATION
      "${fmod_SOURCE_DIR}/lib/macos/libfmod.dylib")
  else ()
    if (ARCH_X86)
      set_property(TARGET fmod PROPERTY IMPORTED_LOCATION
        "${fmod_SOURCE_DIR}/lib/linux/x86_64/libfmod.so.13")
    else ()
      set_property(TARGET fmod PROPERTY IMPORTED_LOCATION
        "${fmod_SOURCE_DIR}/lib/linux/arm64/libfmod.so.13")
    endif ()
  endif ()
endif ()

CPMAddPackage(
  NAME freetype
  URL https://github.com/freetype/freetype/archive/refs/tags/VER-2-12-1.tar.gz
  VERSION 2.12.1
  OPTIONS
  "FT_DISABLE_HARFBUZZ ON"
  "FT_DISABLE_PNG ON"
  "FT_DISABLE_BZIP2 ON"
  "FT_DISABLE_BROTLI ON"
)

CPMAddPackage(
  NAME glew
  URL https://github.com/nigels-com/glew/releases/download/glew-2.2.0/glew-2.2.0.tgz
  VERSION 2.2.0
  SOURCE_SUBDIR build/cmake
  OPTIONS
  "BUILD_UTILS OFF"
)
if (glew_ADDED)
  target_include_directories(glew_s PUBLIC $<BUILD_INTERFACE:${glew_SOURCE_DIR}/include>)
endif ()

CPMAddPackage(
  NAME luajit
  URL https://github.com/LuaJIT/LuaJIT/archive/de2e1ca9d3d87e74c0c20c1e4ad3c32b31a5875b.tar.gz
  VERSION de2e1ca9d3d87e74c0c20c1e4ad3c32b31a5875b
  DOWNLOAD_ONLY TRUE
)
if (luajit_ADDED)
  file(GLOB_RECURSE LUAJIT_SRCS ${luajit_SOURCE_DIR}/src/*.c)
  file(GLOB_RECURSE LUAJIT_HDRS ${luajit_SOURCE_DIR}/src/*.h\(pp\)?)
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
    if("${CMAKE_OSX_DEPLOYMENT_TARGET}" STREQUAL "")
      get_filename_component(osx_sdk_name ${CMAKE_OSX_SYSROOT} NAME)
      string(SUBSTRING ${osx_sdk_name} 6 -1 osx_sdk_version_with_prefix)
      string(LENGTH ${osx_sdk_version_with_prefix} osx_sdk_substr_len)
      math(EXPR osx_sdk_substr_len "${osx_sdk_substr_len}-4")
      string(SUBSTRING ${osx_sdk_version_with_prefix} 0 ${osx_sdk_substr_len} osx_sdk_version)
    else()
      set(osx_sdk_version ${CMAKE_OSX_DEPLOYMENT_TARGET})
    endif()
    message("Passing MACOSX_DEPLOYMENT_TARGET=${osx_sdk_version}")
    add_custom_command(
      COMMAND make amalg MACOSX_DEPLOYMENT_TARGET=${osx_sdk_version} BUILDMODE=static
      COMMAND make install PREFIX=${luajit_BINARY_DIR}
      WORKING_DIRECTORY ${luajit_SOURCE_DIR}
      OUTPUT ${luajit_BINARY_DIR}/lib/${CMAKE_STATIC_LIBRARY_PREFIX}luajit-5.1${CMAKE_STATIC_LIBRARY_SUFFIX}
      DEPENDS ${LUAJIT_SRCS} ${LUAJIT_HDRS}
    )
  else ()
    add_custom_command(
      COMMAND make amalg CFLAGS=-fPIC BUILDMODE=static
      COMMAND make install PREFIX=${luajit_BINARY_DIR}
      WORKING_DIRECTORY ${luajit_SOURCE_DIR}
      OUTPUT ${luajit_BINARY_DIR}/lib/${CMAKE_STATIC_LIBRARY_PREFIX}luajit-5.1${CMAKE_STATIC_LIBRARY_SUFFIX}
      DEPENDS ${LUAJIT_SRCS} ${LUAJIT_HDRS}
    )
  endif ()

  add_library (luajit INTERFACE
    "${luajit_BINARY_DIR}/lib/${CMAKE_STATIC_LIBRARY_PREFIX}luajit-5.1${CMAKE_STATIC_LIBRARY_SUFFIX}")
  target_include_directories (luajit INTERFACE "${luajit_SOURCE_DIR}/src")
  set_property (TARGET luajit PROPERTY INTERFACE_LINK_LIBRARIES
    "${luajit_BINARY_DIR}/lib/${CMAKE_STATIC_LIBRARY_PREFIX}luajit-5.1${CMAKE_STATIC_LIBRARY_SUFFIX}")
endif ()

CPMAddPackage(
  NAME lz4
  URL https://github.com/lz4/lz4/archive/refs/tags/v1.9.4.tar.gz
  VERSION 1.9.4
  SOURCE_SUBDIR build/cmake
  OPTIONS
  "BUILD_SHARED_LIBS OFF"
  "LZ4_BUILD_CLI OFF"
  "LZ4_BUILD_LEGACY_LZ4C OFF"
)

CPMAddPackage(
  NAME sdl
  URL https://github.com/libsdl-org/SDL/releases/download/release-2.26.1/SDL2-2.26.1.tar.gz
  VERSION 2.26.1
  PATCH_COMMAND ${PATCH_TOOL} -p1 -i ${CMAKE_CURRENT_SOURCE_DIR}/cmake/sdl.diff
  OPTIONS
  "SDL_SHARED OFF"
  "SDL_STATIC ON"
  "SDL_TEST OFF"
  "SDL2_DISABLE_INSTALL ON"
  "SDL2_DISABLE_UNINSTALL ON"
  "SDL_STATIC_PIC ON"
)
if (sdl_ADDED)
  target_include_directories(SDL2-static PUBLIC
    $<BUILD_INTERFACE:${sdl_BINARY_DIR}/include>
    $<BUILD_INTERFACE:${sdl_BINARY_DIR}/include-config-$<LOWER_CASE:$<CONFIG>>>)
endif ()

CPMAddPackage(
  NAME stb
  URL https://github.com/nothings/stb/archive/8b5f1f37b5b75829fc72d38e7b5d4bcbf8a26d55.tar.gz
  VERSION 8b5f1f37b5b75829fc72d38e7b5d4bcbf8a26d55
  DOWNLOAD_ONLY TRUE
)
if (stb_ADDED)
  add_library(stb INTERFACE)
  target_include_directories(stb INTERFACE ${stb_SOURCE_DIR})
endif()

if(WIN32)
  CPMAddPackage(
    NAME windirent
    URL https://github.com/tronkko/dirent/archive/328e7fca1497f1d990d8b55b3cec39c869e3a6a8.tar.gz
    VERSION 328e7fca1497f1d990d8b55b3cec39c869e3a6a8
    DOWNLOAD_ONLY TRUE
  )
  if (windirent_ADDED)
    add_library(windirent INTERFACE)
    target_include_directories(windirent INTERFACE ${windirent_SOURCE_DIR}/include)
  endif()
endif()