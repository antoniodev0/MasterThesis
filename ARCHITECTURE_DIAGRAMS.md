# Architettura del Sistema: Diagrammi UML

Questo documento contiene i diagrammi architetturali e UML generati per descrivere la tua tesi. 
Essendo scritti in **Mermaid** (un linguaggio supportato nativamente da GitHub e da VS Code/Cursor), puoi visualizzarli direttamente aprendo l'anteprima Markdown del tuo editor (tasto destro sulla tab del file -> "Open Preview" oppure icona della lente in alto a destra).

---

## 1. Diagramma di Deployment (Impiego e Ambienti)

Questo diagramma mostra *dove* vivono i vari componenti fisici ed emulati, mettendo in risalto la separazione netta tra l'ambiente Cloud (Host Mac) e l'ambiente Edge (Dev Container).

```mermaid
graph TD
    subgraph MAC_HOST ["Mac Host (Cloud Environment)"]
        direction TB
        subgraph DOCKER ["Docker Desktop"]
            direction TB
            subgraph K3D ["k3d Kubernetes Cluster"]
                SPIN_OP["Spin Operator"]
                SHIM["containerd-shim-spin"]
                APP_CLOUD["WebAssembly App (SpinKube)"]
                
                SPIN_OP -.-> SHIM
                SHIM ---> APP_CLOUD
            end
            
            subgraph DEV_CONTAINER ["Dev Container (Ubuntu)"]
                direction TB
                SOCAT["Socat TCP Proxy (10.0.2.2:8080)"]
                
                subgraph QEMU ["QEMU Emulator (x86)"]
                    direction TB
                    subgraph ZEPHYR ["Zephyr RTOS"]
                        WAMR["WAMR (Wasm Micro Runtime)"]
                        APP_EDGE["WebAssembly Payload (.wasm)"]
                        
                        WAMR ---> APP_EDGE
                    end
                end
                ZEPHYR --->|TCP Packet| SOCAT
            end
        end
        
        MAC_HOST_PORT["Port Forwarding (localhost:8080)"]
        SOCAT --->|Forward HTTP| MAC_HOST_PORT
        MAC_HOST_PORT --->|Forward HTTP| K3D
    end
    
    classDef cloud fill:#e1f5fe,stroke:#01579b,stroke-width:2px;
    classDef edge fill:#e8f5e9,stroke:#1b5e20,stroke-width:2px;
    classDef wasm fill:#fff3e0,stroke:#e65100,stroke-width:2px;
    
    class K3D cloud;
    class QEMU edge;
    class APP_CLOUD,APP_EDGE wasm;
```

---

## 2. Diagramma dei Componenti (Logica Applicativa)

Questo diagramma si concentra sull'aspetto del "Write Once, Run Anywhere". Mostra come lo stesso codice sorgente Rust venga compilato in un unico modulo e distribuito in due runtime differenti.

```mermaid
graph LR
    SRC["Codice Sorgente Rust (src/lib.rs)"]
    COMPILER["Compilatore Rust (wasm32-wasip1)"]
    WASM["Binario Base: hello_wasm.wasm"]
    
    SRC --> COMPILER
    COMPILER --> WASM
    
    subgraph CLOUD_RUNTIME ["Cloud Native Runtime"]
        OCI["Immagine OCI (Docker/GHCR)"]
        SPIN["Fermyon Spin HTTP Trigger"]
    end
    
    subgraph EDGE_RUNTIME ["Embedded Runtime"]
        HEADER["Conversione in C Header (.h)"]
        WASI["Chiamate WASI-libc (Socket TCP)"]
    end
    
    WASM -->|Push Registry| OCI
    OCI --> SPIN
    
    WASM -->|xxd script| HEADER
    HEADER --> WASI
```

---

## 3. Diagramma di Sequenza (Comunicazione di Rete)

Questo diagramma UML temporale (Sequence Diagram) mostra esattamente i passaggi logici e temporali della comunicazione TCP End-to-End, dall'avvio della board fino alla risposta HTTP.

```mermaid
sequenceDiagram
    participant Board as Zephyr OS (Edge)
    participant WAMR as WAMR (Runtime)
    participant Wasm as Modulo Wasm (Rust)
    participant Proxy as Socat Proxy
    participant Cloud as K3d SpinKube (Cloud)

    note over Board, WAMR: Boot del Microcontrollore
    Board->>WAMR: Inizializza Memory Pool & Heap
    Board->>WAMR: Imposta WASI Address Pool (Es: 10.0.2.2)
    Board->>WAMR: wasm_runtime_load(hello_wasm_payload)
    Board->>WAMR: wasm_runtime_instantiate()
    
    note over WAMR, Wasm: Esecuzione Sandbox Main
    WAMR->>Wasm: Start esecuzione codice Rust
    Wasm->>WAMR: API WASI syscall: sock_open()
    WAMR-->>Wasm: File Descriptor del Socket
    Wasm->>WAMR: API WASI syscall: sock_connect(10.0.2.2:8080)
    
    note over WAMR, Cloud: Handshake ed Esecuzione Rete
    WAMR->>Proxy: TCP SYN (via QEMU SLIRP)
    Proxy->>Cloud: Forward TCP SYN (localhost:8080)
    Cloud-->>Proxy: TCP SYN-ACK
    Proxy-->>WAMR: TCP SYN-ACK
    WAMR-->>Wasm: Connesso con successo!
    
    Wasm->>WAMR: API WASI syscall: sock_send("GET / HTTP/1.1")
    WAMR->>Proxy: Inoltra Pacchetto TCP (HTTP GET)
    Proxy->>Cloud: Inoltra Pacchetto TCP
    Cloud-->>Proxy: Risposta: HTTP 200 OK ("Hello World!")
    Proxy-->>WAMR: Risposta TCP
    WAMR-->>Wasm: sock_recv() completato
```
