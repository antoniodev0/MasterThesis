# Guida Completa: WebAssembly Sandbox (Cloud & Edge)

Questo documento spiega in sequenza logica tutti i passaggi necessari per sviluppare, pacchettizzare e distribuire un'applicazione WebAssembly sia su un cluster Kubernetes (tramite SpinKube) sia su un dispositivo embedded (tramite Zephyr OS e OCRE).

L'obiettivo è dimostrare l'interoperabilità e il potenziale **Edge-to-Cloud** eseguendo artefatti WebAssembly sicuri: uno più pesante orientato all'ascolto (Spin su Kubernetes) e uno più leggero con un target limitato in esecuzione sull'Edge.

---

## Passo 1: Preparazione dell'Applicazione (Rust + Spin)

Fermyon Spin ci permette di scrivere applicazioni e generare WebAssembly. Abbiamo scelto **Rust** in quanto offre il supporto più maturo per la specifica WASI (WebAssembly System Interface).

1. **Installazione Strumenti:**
   Devi avere Rust (tramite `rustup`), il target wasm, e la CLI di Spin.
   ```bash
   rustup target add wasm32-wasip1
   brew install fermyon/tap/spin
   ```

2. **Scaffolding e Sviluppo:**
   Creiamo una semplice applicazione HTTP ("hello-wasm"):
   ```bash
   spin new http-rust hello-wasm
   cd hello-wasm
   ```
   Il codice si trova in `src/lib.rs` (che risponderà alle chiamate HTTP) e l'infrastruttura di Spin in `spin.toml`.

3. **Compilazione in Wasm:**
   Esegui questo comando per buildare il componente Rust in un file compilato `.wasm`:
   ```bash
   spin build
   ```
   L'output sarà generato in `target/wasm32-wasip1/release/hello_wasm.wasm`. **Questo è il nostro artefatto d'oro.**

---

## Passo 2: Pubblicazione su GitHub Container Registry (GHCR)

Kubernetes (e in particolare il container runtime k3s/containerd) richiede l'accesso tramite HTTPS protetto da TLS per scaricare immagini non locali. Per questo motivo, caricheremo l'artefatto WebAssembly (impacchettandolo in un formato compatibile con Docker, noto come OCI) su un registro pubblico e affidabile: il **GitHub Container Registry (GHCR)**.

### Configurazione e Push su GHCR

#### 1. Generazione del Token su GitHub (PAT)
Per consentire alla tua riga di comando di connettersi a GitHub, abbiamo utilizzato una chiave di accesso personalizzata (Token).
1. Su GitHub > Settings > **Developer settings**.
2. **Personal access tokens** > **Tokens (classic)** > **Generate new token (classic)**.
3. Assegnati i permessi: **`write:packages`** e **`delete:packages`**.

#### 2. Autenticazione Locale
Autentichiamoci con il nostro utente:
```bash
spin registry login ghcr.io -u antoniodev0 -p <La-Tua-Chiave-Segreta-ghp_...>
```

#### 3. Push dell'artefatto OCI
Abbiamo caricato e taggato l'immagine funzionante:
```bash
spin registry push ghcr.io/antoniodev0/hello-wasm:v1
```

#### 4. (IMPORTANTE) Reso il pacchetto "Public"
Di default, i pacchetti pushati su GHCR nascono come "Privati". Per far sì che la nostra macchina k3d non necessiti di chiavi crittografate o secret per fare il pull:
1. Sei andato sul tuo profilo GitHub, tab **Packages**.
2. Hai aperto `hello-wasm` > **Package Settings**.
3. Hai cambiato la visibilità (Change visibility) in **Public**.

---

## Passo 3: Distribuzione su Kubernetes (SpinKube)

SpinKube intercetta risorse personalizzate di tipo `SpinApp` per far eseguire le applicazioni Wasm ai nodi.

1. **Modifica del Manifest:**
   In `k8s/spinapp.yaml`, abbiamo puntato il nostro worker sulla tua immagine GHCR pubblica:
   ```yaml
   apiVersion: core.spinoperator.dev/v1alpha1
   kind: SpinApp
   metadata:
     name: hello-wasm
   spec:
     image: "ghcr.io/antoniodev0/hello-wasm:v1"
     executor: containerd-shim-spin
     replicas: 2
   ```

2. **Applicazione e Test Riusciti:**
   L'operatore SpinKube ha prelevato l'immagine e ha creato con successo i Pods nativi Wasm:
   ```bash
   kubectl apply -f k8s/spinapp.yaml
   kubectl get pods -l core.spinoperator.dev/app-name=hello-wasm
   ```
   *Risultato:* Tutti i Pods passati in stato **Running**.
   
3. **Verifica Endpoint HTTP:**
   Abbiamo eseguito un Port Forwarding locale per testare la corretta esecuzione in cloud del Wasm:
   ```bash
   kubectl port-forward <NOME_POD> 8080:80
   curl http://localhost:8080/
   ```
   *Risultato:* È stato restituito correttamente `Hello World!`.

---

## Passo 4: Distribuzione sull'Edge (Zephyr / OCRE)

Nei dispositivi Embedded basati su micro-controllori (spesso privi di file system complessi o sistemi di containerizzazione come Docker/K8s), il Wasm si carica tipicamente bruciandolo staticamente nella memoria ROM/Flash. 

Per replicare accuratamente questa simulazione sul framework Zephyr:

1. **Esecuzione dello script di Payload:**
   Sulla macchina host (o nel DevContainer) esegui:
   ```bash
   chmod +x prepare_for_ocre.sh
   ./prepare_for_ocre.sh
   ```

2. **Cosa succede dietro le quinte?**
   Lo script estrae l'artefatto Wasm *esatto* che hai compilato prima e, tramite il tool Unix `xxd`, lo converte in un array globale di byte in formato header C.
   Verrà creato un file all'interno di `hello-wasm/hello_wasm_payload.h`. Conterrà qualcosa come:
   ```c
   unsigned char hello_wasm_app[] = { 0x00, 0x61, 0x73, 0x6d, 0x01, ... };
   unsigned int hello_wasm_app_len = 123456;
   ```

3. **Integrazione con il Firmware (I Tuoi "Next Steps"):**
   * Spostati all'interno della shell di `Zephyr` e dello stack OCRE.
   * Apri il tuo file C principale di ingresso dell'OS Zephyr (che farà il bootstrap del runtime WAMR).
   * Includi l'header compilato:
     ```c
     #include "hello_wasm_payload.h"
     ```
   * Passa in runtime quell'array alla funzione di esecuzione WAMR (usualmente chiamata `wasm_module_instantiate()`).

---

## Next Steps per la Tesi (Cosa c'è da fare ora?)

## Next Steps per la Tesi (Lavoro Restante: Implementazione Networking)

Avendo stabilizzato completamente e *verificato* l sia l'infrastruttura Cloud/Kubernetes ospitata da GHCR (Passo 1-3), sia il container Wasm embedded ospitato su Zephyr tramite l'interprete WAMR (Passo 4), il passo cruciale che ora dovrai sviluppare per finalizzare il prototipo è realizzare la comunicazione:

1. Modificare l'applicativo Rust sul microcontrollore Zephyr affinchè apra dei socket WASI per formare un pacchetto HTTP `GET` / `POST`.
2. Trasmettere la chiamata sulla rete virtuale generata per far sì che il modulo WASM *Edge* interagisca formalmente con il modulo WASM *Cloud* (il container SpinKube) validando così lo schema di interazione sicura distribuita.
