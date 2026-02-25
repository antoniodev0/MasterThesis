#include <zephyr/kernel.h>
#include <stdio.h>
#include <stdlib.h>

// WAMR headers (make sure WAMR is properly fetched via west)
#include "wasm_export.h"

// Include the Wasm payload we generated via xxd!
#include "hello_wasm_payload.h"

int main(void) {
    printk("===============================================\n");
    printk(" OCRE Wasm Sandbox - Master Thesis Application \n");
    printk("===============================================\n");

    printk("\nPayload Info:\n");
    printk("- Wasm Size: %d bytes\n", hello_wasm_app_len);
    printk("- Wasm Magic Header: 0x%02x 0x%02x 0x%02x 0x%02x\n\n", 
           hello_wasm_app[0], hello_wasm_app[1], hello_wasm_app[2], hello_wasm_app[3]);

    /*
     * INTERGRATION TODO FOR MASTER THESIS:
     * 1. Ensure Zephyr's west workspace has pulled the WAMR module.
     * 2. Uncomment the WASM runtime initialization below.
     * 3. Compile and Run via `west build` in the Dev Container.
     */

    static char global_heap_buf[2 * 1024 * 1024];
    RuntimeInitArgs init_args;
    memset(&init_args, 0, sizeof(RuntimeInitArgs));

    init_args.mem_alloc_type = Alloc_With_Pool;
    init_args.mem_alloc_option.pool.heap_buf = global_heap_buf;
    init_args.mem_alloc_option.pool.heap_size = sizeof(global_heap_buf);

    if (!wasm_runtime_full_init(&init_args)) {
        printk("Init runtime environment failed.\n");
        return -1;
    }

    char error_buf[128];
    wasm_module_t module = wasm_runtime_load(hello_wasm_app, hello_wasm_app_len, error_buf, sizeof(error_buf));
    if (!module) {
        printk("Load wasm module failed. Error: %s\n", error_buf);
        return -1;
    }

    // Pass address pool to allow WASI Socket external connections
    // The format is "addr/mask". We allow the entire local subnet.
    const char *addr_pool[1] = { "10.0.2.0/24" };

    wasm_runtime_set_wasi_args(module, NULL, 0, NULL, 0, NULL, 0, NULL, 0);
    wasm_runtime_set_wasi_addr_pool(module, addr_pool, 1);

    wasm_module_inst_t module_inst = wasm_runtime_instantiate(module, 64 * 1024, 128 * 1024, error_buf, sizeof(error_buf));
    if (!module_inst) {
        printk("Instantiate wasm module failed. Error: %s\n", error_buf);
        return -1;
    }

    printk("Executing Wasm module...\n");
    if (!wasm_application_execute_main(module_inst, 0, NULL)) {
        printk("Execution failed. Error: %s\n", wasm_runtime_get_exception(module_inst));
    } else {
        printk("Execution successful.\n");
    }
    
    // Cleanup
    wasm_runtime_deinstantiate(module_inst);
    wasm_runtime_unload(module);
    wasm_runtime_destroy();

    printk("\nReady for Zephyr Runtime Execution.\n");
    return 0;
}
