# Configurazione: SpinKube & OCRE

Questo documento descrive i passi esatti eseguiti per configurare l'ambiente di ricerca per la tesi magistrale.

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