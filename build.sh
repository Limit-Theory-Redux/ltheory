#!/bin/bash
set -e

debug=0
if [[ $1 == "debug" ]]; then
    debug=1
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
    if [[ -d "/c/Program Files/LLVM/bin" ]]; then
        export LIBCLANG_PATH="/c/Program Files/LLVM/bin"
    elif [[ -d "/c/Program Files (x86)/LLVM/bin" ]]; then
        export LIBCLANG_PATH="/c/Program Files (x86)/LLVM/bin"
    else
        echo "Please the environment variable LIBCLANG_PATH to the path containing clang.exe in your LLVM installation directory, i.e. C:\Program Files\LLVM\bin"
        exit 1
    fi
fi

if [[ $debug = 1 ]]; then
    cargo build
    cp target/debug/ltr${binsuffix} bin/lt64d${binsuffix}
    cp target/debug/deps/${libprefix}phx${libsuffix} bin/${libprefix}phx${libsuffix}
    cp target/debug/deps/${libprefix}fmod${libsuffix} bin/${libprefix}fmod${libsuffix}
else
    cargo build --release
    cp target/release/ltr${binsuffix} bin/lt64${binsuffix}
    cp target/release/deps/${libprefix}phx${libsuffix} bin/${libprefix}phx${libsuffix}
    cp target/release/deps/${libprefix}fmod${libsuffix} bin/${libprefix}fmod${libsuffix}
fi
