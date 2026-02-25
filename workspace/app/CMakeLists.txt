cmake_minimum_required(VERSION 3.20.0)

# Initialize Zephyr
find_package(Zephyr REQUIRED HINTS $ENV{ZEPHYR_BASE})
project(wasm_embedded_app)

# WAMR Config: Required settings for WASM Micro Runtime
set(WAMR_BUILD_TARGET "X86_32" CACHE STRING "Target architecture")
set(WAMR_BUILD_AOT 0 CACHE BOOL "Disable AOT (we use JIT/Interpreter for Wasm)")
set(WAMR_BUILD_INTERP 1 CACHE BOOL "Enable Interpreter")
set(WAMR_BUILD_FAST_INTERP 1 CACHE BOOL "Enable Fast Interpreter")
set(WAMR_BUILD_WASI_MINCORE 1 CACHE BOOL "Enable WASI Mincore")
set(WAMR_BUILD_REF_TYPES 1 CACHE BOOL "Enable Reference Types")
set(WAMR_BUILD_LIBC_WASI 1 CACHE BOOL "Enable WASI")
set(WAMR_BUILD_LIBC_WASI_NETWORK 1 CACHE BOOL "Enable WASI Socket Networking")
set(WAMR_BUILD_LIBC_BUILTIN 1 CACHE BOOL "Enable Builtin libc")
if(DEFINED ENV{WAMR_BASE})
    set(WAMR_DIR $ENV{WAMR_BASE})
else()
    # Fallback to typical Zephyr module path if WAMR is fetched via west
    set(WAMR_DIR ${ZEPHYR_BASE}/../modules/lib/wamr)
endif()

# Include WAMR core if it exists
if(EXISTS ${WAMR_DIR}/build-scripts)
    set(WAMR_BUILD_PLATFORM "zephyr")
    set(WAMR_ROOT_DIR ${WAMR_DIR})
    include(${WAMR_DIR}/build-scripts/runtime_lib.cmake)
else()
    message(WARNING "WAMR directory not found at ${WAMR_DIR}. Make sure 'west update' has fetched it.")
endif()

# Add our application source code
target_sources(app PRIVATE ${WAMR_RUNTIME_LIB_SOURCE} src/main.c)

# Include the directory containing our generated `hello_wasm_payload.h`
target_include_directories(app PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/../../hello-wasm)
