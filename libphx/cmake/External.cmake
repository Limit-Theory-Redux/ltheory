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

if(LINUX)
  set(OpenGL_GL_PREFERENCE GLVND)
endif()
set(CMAKE_REQUIRED_QUIET ON)
set(X11_FIND_QUIETLY ON)

CPMAddPackage(
  NAME Bullet
  URL https://github.com/bulletphysics/bullet3/archive/refs/tags/3.24.tar.gz
  VERSION 3.24
  PATCH_COMMAND ${PATCH_TOOL} -p1 -i ${CMAKE_CURRENT_SOURCE_DIR}/cmake/Bullet.diff
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
if (Bullet_ADDED)
  target_include_directories(BulletDynamics PUBLIC ${Bullet_SOURCE_DIR}/src)
endif ()
