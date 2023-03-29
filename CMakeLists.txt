cmake_minimum_required (VERSION 3.0.2)
project (LTheory)
include (libphx/cmake/Shared.cmake)

add_subdirectory (libphx)

add_executable (lt "src/Main.cpp")
phx_configure_output_dir (lt)
phx_configure_target_properties (lt)

if (WIN32)
  target_sources(lt PRIVATE "src/lt.manifest")
endif ()

target_compile_definitions (lt PRIVATE DEBUG=$<CONFIG:DEBUG>)
target_link_libraries (lt phx)

set_target_properties (lt PROPERTIES
  OUTPUT_NAME_DEBUG "lt${ARCH}d"
  OUTPUT_NAME_RELEASE "lt${ARCH}"
  OUTPUT_NAME_RELWITHDEBINFO "lt${ARCH}"
  OUTPUT_NAME_MINSIZEREL "lt${ARCH}")

if (WIN32)
  set_property(TARGET lt PROPERTY VS_DEBUGGER_WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}")
endif ()
