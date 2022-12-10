if (WIN32)
  set (PLATFORM "win")
  set (WINDOWS TRUE)
elseif (UNIX AND NOT APPLE)
  set (PLATFORM "linux")
  set (LINUX TRUE)
elseif (APPLE)
  set (PLATFORM "macos")
  set (MACOS TRUE)
else ()
  message (FATAL_ERROR "Unsupported Platform")
endif ()

string(TOLOWER ${CMAKE_SYSTEM_PROCESSOR} system_processor)
if ("${CMAKE_SIZEOF_VOID_P}" EQUAL "4")
  set (ARCH "32")
elseif ("${CMAKE_SIZEOF_VOID_P}" EQUAL "8")
  set (ARCH "64")
else ()
  message (FATAL_ERROR "Unsupported CPU pointer size ${CMAKE_SIZEOF_VOID_P} (either 32 or 64 bit supported)")
endif ()

if ("${system_processor}" MATCHES "(x86)|(amd64)")
  set (ARCH_X86 TRUE)
elseif ("${system_processor}" STREQUAL "arm64")
  set (ARCH_ARM TRUE)
else ()
  message (FATAL_ERROR "Unsupported CPU Architecture: ${system_processor}")
endif()

set (PLATARCH "${PLATFORM}${ARCH}")

set(default_build_type RelWithDebInfo)
if(NOT CMAKE_BUILD_TYPE AND NOT CMAKE_CONFIGURATION_TYPES)
  message(STATUS "Setting build type to '${default_build_type}' as none was specified.")
  set(CMAKE_BUILD_TYPE "${default_build_type}" CACHE STRING "Choose the type of build." FORCE)
  set_property(CACHE CMAKE_BUILD_TYPE PROPERTY STRINGS "Debug" "Release" "MinSizeRel" "RelWithDebInfo")
endif()

function (phx_configure_output_dir target)
  set_target_properties (${target} PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_SOURCE_DIR}/bin"
    LIBRARY_OUTPUT_DIRECTORY "${CMAKE_SOURCE_DIR}/bin"
    ARCHIVE_OUTPUT_DIRECTORY "${CMAKE_SOURCE_DIR}/bin")

  foreach (config ${CMAKE_CONFIGURATION_TYPES})
    string (TOUPPER ${config} config)
    set_target_properties (${target} PROPERTIES
      RUNTIME_OUTPUT_DIRECTORY_${config} "${CMAKE_SOURCE_DIR}/bin"
      LIBRARY_OUTPUT_DIRECTORY_${config} "${CMAKE_SOURCE_DIR}/bin"
      ARCHIVE_OUTPUT_DIRECTORY_${config} "${CMAKE_SOURCE_DIR}/bin")
  endforeach (config)
endfunction ()

function (phx_configure_target_properties target)
  if (WINDOWS)
    target_compile_definitions (${target} PRIVATE _CRT_SECURE_NO_DEPRECATE)
    target_compile_definitions (${target} PRIVATE WIN32_LEAN_AND_MEAN)
    target_compile_definitions (${target} PRIVATE WINDOWS=1)

    target_compile_options (${target} PRIVATE "/MP")         # Multithreaded Build
    target_compile_options (${target} PRIVATE "/MD")         # Dynamic C Runtime
    target_compile_options (${target} PRIVATE "/EHs-c-")     # No exception handling
    target_compile_options (${target} PRIVATE "/fp:fast")    # No strict FP
    target_compile_options (${target} PRIVATE "/GL")         # Whole Program Optimization
    target_compile_options (${target} PRIVATE "/GS-")        # No Buffer Security Checks
    target_compile_options (${target} PRIVATE "/GR-")        # No RTTI
    target_compile_options (${target} PRIVATE "/arch:SSE2")  # Assume SSE2+
  elseif (LINUX OR MACOS)
    set_property(TARGET ${target} PROPERTY BUILD_WITH_INSTALL_RPATH ON)
    if(LINUX)
      set_property(TARGET ${target} PROPERTY INSTALL_RPATH "\$ORIGIN")
    else()
      set_property(TARGET ${target} PROPERTY INSTALL_RPATH "@executable_path")
    endif()

    target_compile_definitions (${target} PRIVATE UNIX=1)

    target_compile_options (${target} PRIVATE "-Wall")            # All error checking
    target_compile_options (${target} PRIVATE "-fno-exceptions")  # No exception handling
    target_compile_options (${target} PRIVATE "-ffast-math")      # No strict FP
    target_compile_options (${target} PRIVATE "-fpic")            # PIC since this is shared

    target_compile_options (${target} PRIVATE "-Wno-unused-variable")
    target_compile_options (${target} PRIVATE "-Wno-unknown-pragmas")

    target_compile_options (${target} PRIVATE "-std=c++11")
  endif ()
endfunction ()
