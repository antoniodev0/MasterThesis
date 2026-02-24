# Guida: Esecuzione di WebAssembly Sandbox sull'Edge (Zephyr / OCRE)

Questo documento spiega i passaggi operativi da seguire **all'interno del Dev Container** per compilare ed eseguire un modulo WebAssembly (`hello-wasm`) sul sistema operativo per microcontrollori Zephyr RTOS, sfruttando WAMR (WebAssembly Micro Runtime).

---

## Panoramica dell'Architettura

### Il problema del "Component Model" e gli Shims WASM
Inizialmente, l'applicazione `hello-wasm` era stata creata sfruttando lo **Spin SDK** per essere caricata su cluster Kubernetes o orchestratori Cloud. Lo Spin SDK compila il codice in un modulo orientato al web/backend (un *WASI Reactor* o *HTTP Component*) includendo l'importazione di numerose interfacce host molto complesse (es. `wasi:http/types`, `wasi:io/poll`).

Tuttavia, queste interfacce si aspettano un "host" molto ricco che fornisca tutte queste implementazioni sottostanti. Quando abbiamo tentato di eseguire questo stesso payload su un microcontrollore (tramite Zephyr RTOS e l'interprete WAMR), il runtime non è riuscito a collegare tutte queste importazioni perché mancavano le relative **Host Functions** (spesso chiamati "shims").

Gli *shims WASM* sono essenzialmente delle "funzioni ponte" scritte in linguaggio nativo (es. C o C++) esposte dall'ambiente host (in questo caso Zephyr) per interfacciarsi con il sistema operativo reale e le risorse fisiche o emulativi del device (es. networking, file system, timer). Senza fornire l'esatta implementazione per tutti gli shims richiesti dallo Spin SDK tramite le API corrette, il modulo fa fallire la fase di "Instanziazione" WAMR o va in crash lamentando proprio la mancanza di *module import*. Oltre agli shims, WAMR su RTOS normalmente si aspetta una build che esponga la tradizionale funzione entry point `_start` per i processi WASM (*WASI Command* standards).

### Soluzione Adottata
Per ovviare a questo problema concettuale ed eseguire correttamente il codice direttamente sull'Edge/RTOS "nudo", abbiamo scollegato il codice originale Rust dallo Spin SDK convertendolo in un *WASI Command* standard da linea di comando:
1. Modificato da modulo libreria (`lib.rs` / `cdylib`) in un normale eseguibile standalone (`main.rs` con la funzione base `main() { ... }`).
2. Rimosso ogni riferimento logico e di dipendenza a Spin SDK.
3. Compilato usando il target neutrale `wasm32-wasip1`.

Così facendo, il file WebAssembly (.wasm) generato interagisce unicamente con le primitive e le dipendenze elementari di libc standard (per stampare su stdout) sopportate nativamente e senza mapping custom dalle impostazioni di default che noi abbiamo attivato in Zephyr.

---

## Sequenza Operativa Logica (Step-by-Step)

### Step 1: Compilazione o Aggiornamento del Modulo WebAssembly (Rust)
Dal terminale del Dev Container (che possiede i runtime preinstallati per la tua tesi), entra nella cartella `hello-wasm` e compila il binario puro specificando l'architettura WASI base adatta per WAMR:

```bash
cd /workspaces/MasterThesis/hello-wasm

# (Solo la prima volta) Installare target WebAssembly
rustup target add wasm32-wasip1

cargo build --target wasm32-wasip1 --release
```
**Risultato:** Verrà generato il file binario `hello-wasm/target/wasm32-wasip1/release/hello-wasm.wasm`.

### Step 2: Creazione Payload (Header C)
Zephyr compila il firmware in un target in base C (C/C++). Avendo bisogno di inglobare questo eseguibile WebAssembly direttamente nella flash o in memoria del microcontrollore in fase iniziale di start per farlo eseguire a WAMR, decodifichiamo l'artefatto usando lo script automatizzato.

```bash
cd /workspaces/MasterThesis
./prepare_for_ocre.sh
```
**Risultato:** Il payload convertito si trova ora come array di byte in `hello-wasm/hello_wasm_payload.h` con variabile C esportata come `hello_wasm_app`.

### Step 3: Fetch dei Moduli Zephyr
Assicurati che l'infrastruttura base e i moduli SDK (come quello ufficiale open-source di *wasm-micro-runtime* specificato nel `west.yml`) siano aggiornati per supportare il mapping:

```bash
cd /workspaces/MasterThesis/workspace
west update
```

### Step 4: Compilazione Finale del Firmware Embedded
Lanciando la build passiamo contemporaneamente i parametri e le variabili di modulo integrate in `app/src/main.c`, i requirements in `app/prj.conf` (stack, WASI posix networking, littlefs) e `app/CMakeLists.txt`. In assenza di un device hardware colleghiamo tutto in un emulatore (`qemu_x86`).

```bash
cd /workspaces/MasterThesis/workspace
# Il flag -p always forza una pulitura (clean) essenziale prima della compilazione su Zephyr
west build -p always -b qemu_x86 app/
```

### Step 5: Test della Sandbox ed Esecuzione in Emulazione
Lancia l'avvio della piattaforma emulata QEMU. Il processo virtuale isolerà il firmware in un sandbox runtime sicuro:

```bash
west build -t run
```

### Risultato Ottenuto a Video
Sul terminale comparirà il log del boot process di Zephyr OS e immediatamente dopo il print di base invocato internamente dal WebAssembly:

```text
===============================================
 OCRE Wasm Sandbox - Master Thesis Application 
===============================================

Payload Info:
- Wasm Size: 64328 bytes
- Wasm Magic Header: 0x00 0x61 0x73 0x6d

Executing Wasm module...
Hello World!
Execution successful.

Ready for Zephyr Runtime Execution.
```
