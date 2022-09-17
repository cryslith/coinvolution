#!/bin/bash

set -euo pipefail

cd "$(dirname "$0")"

rm -rf pkg
wasm-pack build --debug --target web -- --features console_error_panic_hook
cp js/* pkg/
