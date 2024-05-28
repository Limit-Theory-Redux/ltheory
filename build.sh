#!/bin/bash
set -e

source ./utils.sh

run_tests=false
debug=false
bundle=false
universal=false

while test $# -gt 0; do
  case "$1" in
    -h|--help)
      echo "build.sh - Builds and packages Limit Theory Redux."
      echo " "
      echo "options:"
      echo "-h, --help          show brief help"
      echo "    --run-tests     run tests"
      echo "    --debug         build and run tests in debug mode"
      echo "    --bundle        assemble an app bundle (only has an effect on macOS)"
      exit 0
      ;;
    --run-tests)
      run_tests=true
      shift
      ;;
    --debug)
      debug=true
      shift
      ;;
    --bundle)
      bundle=true
      shift
      ;;
    *)
      echo "Unknown flag $1"
      exit 1
      ;;
  esac
done

# Tests are currently not working correctly on Linux.
if [[ $run_tests == true && "$OSTYPE" == "linux-gnu"* ]]; then
    echo "Tests are currently not working correctly on Linux, disabling."
    run_tests=false
fi

if [[ "$OSTYPE" == "darwin"* ]]; then
    libprefix="lib"
    libsuffix=".dylib"
    binsuffix=""
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    libprefix="lib"
    libsuffix=".so"
    binsuffix=""
elif [[ "$OSTYPE" == "msys" ]]; then
    libprefix=""
    libsuffix=".dll"
    binsuffix=".exe"
    if [[ -z "$LIBCLANG_PATH" ]]; then
        if [[ -d "/c/Program Files/LLVM/bin" ]]; then
            export LIBCLANG_PATH="/c/Program Files/LLVM/bin"
        elif [[ -d "/c/Program Files (x86)/LLVM/bin" ]]; then
            export LIBCLANG_PATH="/c/Program Files (x86)/LLVM/bin"
        else
            echo "Set the environment variable LIBCLANG_PATH to the path containing clang.exe in your LLVM installation directory, i.e. C:\Program Files\LLVM\bin"
            exit 1
        fi
    fi
fi

if [ -z "$PHX_VERSION" ]; then
    export PHX_VERSION
fi

if [[ $debug == true ]]; then
    target_dir="target/debug"

    cargo build --color auto

    if [[ $run_tests == true ]]; then
        cargo test --no-fail-fast --color auto
    fi
else
    target_dir="target/release"

    cargo build --release --color auto

    if [[ $run_tests == true ]]; then
        cargo test --release --no-fail-fast --color auto
    fi
fi

# Re-populate the 'bin' directory.
rm -rf bin && mkdir -p bin

cp "${target_dir}/ltr${binsuffix}" "bin/ltr${binsuffix}"
cp "${target_dir}/deps/${libprefix}phx${libsuffix}" "bin/${libprefix}phx${libsuffix}"

if [[ $bundle == true ]]; then
    create_app_bundle bin
fi
