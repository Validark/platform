#!/usr/bin/env bash

TARGET=wasm32-unknown-unknown

PROFILE_ARG=""
PROFILE="debug"

if [ -n "$CARGO_BUILD_PROFILE" ]; then
  if [ "$CARGO_BUILD_PROFILE" == "release" ]; then
    PROFILE_ARG="--release"
    PROFILE="release"
  elif [ "$CARGO_BUILD_PROFILE" != "dev" ]; then
    PROFILE_ARG="--profile $CARGO_BUILD_PROFILE"
    PROFILE="$CARGO_BUILD_PROFILE"
  fi
fi

OUTPUT_DIR="$PWD/wasm"
OUTPUT_FILE="$OUTPUT_DIR/wasm_dpp_bg.wasm"
BUILD_COMMAND="cargo build --config net.git-fetch-with-cli=true --target=$TARGET $PROFILE_ARG"
BINDGEN_COMMAND="wasm-bindgen --out-dir=$OUTPUT_DIR --target=web --omit-default-module-path ../../target/$TARGET/$PROFILE/wasm_dpp.wasm"

if ! [[ -d $OUTPUT_DIR ]]; then
  mkdir -p "$OUTPUT_DIR"
fi

if ! [[ -x "$(command -v wasm-bindgen)" ]]; then
  echo 'Wasm-bindgen CLI is not installed. Installing'
  cargo install --config net.git-fetch-with-cli=true -f wasm-bindgen-cli
fi

# On a mac, bundled clang won't work - you need to install LLVM manually through brew,
# and then set the correct env for the build to work
if [[ "$OSTYPE" == "darwin"* ]]; then
  AR_PATH=$(which llvm-ar)
  CLANG_PATH=$(which clang)
  AR=$AR_PATH CC=$CLANG_PATH $BUILD_COMMAND
  AR=$AR_PATH CC=$CLANG_PATH $BINDGEN_COMMAND
else
  $BUILD_COMMAND
  $BINDGEN_COMMAND
fi

# EMCC_CFLAGS="-s ERROR_ON_UNDEFINED_SYMBOLS=0 --no-entry" cargo build --target=wasm32-unknown-emscripten --release
# EMCC_CFLAGS="-s ERROR_ON_UNDEFINED_SYMBOLS=0 --no-entry" wasm-bindgen --out-dir=wasm --target=web --omit-default-module-path ../../target/wasm32-unknown-emscripten/release/wasm_dpp.wasm

# TODO: Must be somehow preinstalled?
#if [ "$PROFILE" == "release" ]; then
#  echo "Optimizing wasm using Binaryen"
#  wasm-opt -Os "$OUTPUT_FILE" -o "$OUTPUT_FILE"
#fi
