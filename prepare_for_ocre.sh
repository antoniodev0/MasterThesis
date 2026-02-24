#!/bin/bash
set -e

echo "Preparing WebAssembly payload for OCRE (Zephyr)..."

WASM_FILE="hello-wasm/target/wasm32-wasip1/release/hello-wasm.wasm"
DEST_FILE="hello-wasm/hello_wasm_payload.h"

if [ ! -f "$WASM_FILE" ]; then
    echo "Error: $WASM_FILE not found. Did you run 'spin build'?"
    exit 1
fi

echo "Converting .wasm binary to C array using xxd..."
xxd -i "$WASM_FILE" | sed 's/hello_wasm_target_wasm32_wasip1_release_hello_wasm_wasm/hello_wasm_app/g' > "$DEST_FILE"

echo "Success! The Wasm payload has been written to $DEST_FILE"
echo "You can now include this header in your Zephyr/OCRE main.c:"
echo ""
echo "#include \"hello_wasm_payload.h\""
echo "/* Pass hello_wasm_app and hello_wasm_app_len to the WAMR runtime initialization */"
