# Tesi Magistrale: Edge-to-Cloud WebAssembly Sandbox (OCRE & SpinKube)

Questo documento descrive l'architettura e i passi esatti eseguiti per configurare l'ambiente di ricerca per la tesi magistrale, focalizzata sull'interoperabilità di moduli WebAssembly tra Cloud ed Edge.

## 1. Setup dell'Infrastruttura Kubernetes

L'obiettivo è creare un cluster locale che supporti nativamente carichi di lavoro WebAssembly.

### 1.1 Creazione del Cluster con k3d

Abbiamo creato un cluster Kubernetes simulato in Docker che include già lo shim per WebAssembly.

```bash
k3d cluster create wasm-cluster \
  --image ghcr.io/spinkube/containerd-shim-spin/k3d:v0.15.1 \
  --port "8081:80@loadbalancer" \
  --agents 2
```

**Spiegazione:** Usiamo un'immagine nodo custom (`ghcr.io/.../k3d`) perché contiene il binario `containerd-shim-spin`. Questo componente è un "interprete" che intercetta le chiamate di Kubernetes e, invece di avviare un container Linux tramite `runc`, avvia il modulo Wasm tramite Wasmtime.

### 1.2 Installazione di Cert-Manager

Prerequisito fondamentale per la sicurezza interna degli operatori Kubernetes.

```bash
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.14.3/cert-manager.yaml
```

**Perché:** L'operatore di SpinKube utilizza dei "Webhook" per validare le risorse (CRD). Questi Webhook richiedono certificati TLS validi per comunicare con l'API Server di Kubernetes.

### 1.3 Configurazione delle Custom Resource Definitions (CRD) di Spin

Le CRD istruiscono Kubernetes su come gestire i nuovi "oggetti" relativi a WebAssembly.

```bash
# Installazione delle definizioni delle risorse
kubectl apply -f https://github.com/spinkube/spin-operator/releases/download/v0.3.0/spin-operator.crds.yaml

# Installazione della RuntimeClass
kubectl apply -f https://github.com/spinkube/spin-operator/releases/download/v0.3.0/spin-operator.runtime-class.yaml
```

**RuntimeClass:** Definisce un nuovo tipo di runtime nel cluster chiamato `wasmtime-spin-v2`. Quando un Pod specifica questa classe, Kubernetes sa che non deve usare Docker/containerd standard.

### 1.4 Installazione dello Spin Operator tramite Helm

L'operatore è il "controller" che osserva il cluster e reagisce quando l'utente crea una SpinApp.

```bash
helm install spin-operator \
  --namespace spin-operator \
  --create-namespace \
  --version 0.3.0 \
  --wait \
  oci://ghcr.io/spinkube/charts/spin-operator
```

**OCI Registry:** Abbiamo usato `oci://` perché Helm ora supporta la distribuzione dei pacchetti (Charts) come immagini Docker standard.

### 1.5 Attivazione dello Shim Executor

Infine, colleghiamo l'operatore all'effettivo esecutore sui nodi.

```bash
kubectl apply -f https://github.com/spinkube/spin-operator/releases/download/v0.3.0/spin-operator.shim-executor.yaml
```

**Nota:** Questo comando crea una risorsa `SpinAppExecutor`. Serve a mappare l'intento logico (eseguire un'app) con il binario fisico (`containerd-shim-spin`) presente sui nodi k3d.

## 2. Setup dell'Ambiente Embedded (Dev Container)

Ambiente isolato per la compilazione del firmware e del runtime OCRE (basato su Zephyr RTOS).

### 2.1 Configurazione `.devcontainer/devcontainer.json`

Abbiamo configurato Cursor per avviare un container Linux Ubuntu con la toolchain Zephyr.

**Correzione Architetturale per macOS M3:**
Inizialmente abbiamo provato a usare `--userns=keep-id`, ma è stato rimosso poiché Docker Desktop su Mac gestisce i permessi tramite virtualizzazione (virtiofs) e quel flag causava il fallimento del montaggio dei volumi.

### 2.2 Inizializzazione di Zephyr (West)

All'interno del container, abbiamo eseguito i comandi per scaricare il kernel RTOS:

```bash
# Inizializza il meta-repository di Zephyr
west init -m https://github.com/zephyrproject-rtos/zephyr --mr main workspace

# Scarica tutti i moduli (HAL, Driver, Network Stack)
cd workspace
west update

# Esporta la configurazione per i tool di build (CMake)
west zephyr-export
```

**West:** È il tool di gestione di Zephyr. È necessario perché Zephyr non è un singolo blocco di codice, ma una collezione di centinaia di repository (uno per ogni produttore di chip: STM32, Nordic, NXP, ecc.).

## 3. Verifica dello Stato Attuale

Per confermare che tutto sia configurato correttamente per la fase di sviluppo:

*   **Stato Cluster:** `kubectl get nodes` -> 3 nodi Ready (1 Server, 2 Agents).
*   **Stato Operatore:** `kubectl get pods -n spin-operator` -> Pods in Running.
*   **Stato Embedded:** Terminale Cursor connesso al container con `west` versione 1.5.0

## 4. Architettura Edge-to-Cloud (WebAssembly)

Una volta preparato l'ambiente, abbiamo delineato l'architettura applicativa per dimostrare l'interoperabilità strutturale di WebAssembly tra Cloud ed Edge. 

### 4.1 Il Cloud (SpinKube / Kubernetes)
Sfruttando **Fermyon Spin**, abbiamo sviluppato un componente orientato al backend (un *WASI Reactor* o *HTTP Component*). Questo modulo WebAssembly è stato:
- Compilato in formato `.wasm`.
- Impacchettato come immagine OCI e caricato su GitHub Container Registry (GHCR).
- Distribuito sul cluster Kubernetes locale tramite la CRD `SpinApp` (`k8s/spinapp.yaml`).
In questo strato, l'infrastruttura Cloud espone il web server, scala e riceve richieste HTTP in maniera efficiente, demandando la complessità del networking agli Shims di SpinKube.

### 4.2 L'Edge (Zephyr / OCRE)
Per l'Edge (microcontrollori e RTOS), abbiamo affrontato la limitazione tecnica nativa degli shims WASM. Non potendo ospitare un intero server Spin su Zephyr, abbiamo:
1. Sviluppato un modulo Rust standalone trasformato in standard compilato `wasm32-wasip1` (*WASI Command*).
2. Convertito il binario in un header C (`hello_wasm_payload.h`) tramite lo script automatizzato.
3. Incorporato il payload nel firmware Zephyr, consentendo al runtime integrato (WAMR) di eseguirlo nativamente in una Sandbox con accesso diretto alla memoria heap base.

Il modulo Wasm lato Edge agisce quindi da "client", leggero e isolato, rispetto al server pesante sul Cloud.

## 5. Next Steps della Tesi

Attualmente, l'architettura di base è completa: il Cloud esegue con successo moduli HTTP Wasm e l'Edge emulato su QEMU o hardware fisico esegue in Sandbox moduli standard Wasm tramite Zephyr.

**L'obiettivo finale e prossimo passo sarà abilitare il Networking:**
Il codice Rust implementato sul microcontrollore Zephyr dovrà aprire un Socket di rete standard WASI (TCP), formattare una richiesta HTTP (es. GET/POST) e *chiamare* l'endpoint esposto dal cluster SpinKube. In questo modo si concretizzerà la comunicazione protetta Edge-to-Cloud demandata esclusivamente a moduli WebAssembly "universali".

Per la guida dettagliata passo-passo sui due ambienti, consultare:
- [Guida Cloud: WASM_SANDBOX_GUIDE.md](WASM_SANDBOX_GUIDE.md)
- [Guida Edge: ZEPHYR_OCRE_GUIDE.md](ZEPHYR_OCRE_GUIDE.md)