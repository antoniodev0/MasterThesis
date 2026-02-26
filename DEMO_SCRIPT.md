# Demo Architettura Edge-to-Cloud WebAssembly

Questa è la guida "copione" per mostrare al tuo professore il funzionamento end-to-end del tuo progetto di tesi. Seguendo questi passaggi, mostrerai come un modulo WebAssembly (Wasm) in esecuzione su un ambiente Edge ultra-vincolato (Zephyr OS) riesce a comunicare con un altro modulo WebAssembly scalabile ospitato sul Cloud (Kubernetes).

---

## Preparazione (Prima della Demo)

Assicurati che l'ambiente sia acceso e pronto per evitare tempi morti durante la presentazione:
1. Docker Desktop è avviato.
2. Il cluster `k3d` Kubernetes locale è in esecuzione (`docker ps` mostra i nodi `k3d-wasm-cluster-server-0`, ecc.).
3. Sei connesso al Dev Container di Zephyr su VS Code/Cursor.

---

## Step 1: Mostrare il Backend Cloud (SpinKube)

**Obiettivo:** Dimostrare che il Cloud è pronto a ricevere le misurazioni o i messaggi dall'Edge.

1. **Apri un terminale (fuori dal Dev Container, sul tuo Mac) e mostra i Pod Kubernetes:**
   ```bash
   kubectl get pods -l core.spinoperator.dev/app-name=hello-wasm
   ```
   *Spiega al professore:* "Questi Pod non stanno eseguendo container di Linux standard (Docker), ma moduli WebAssembly nudi, orchestrati tramite lo Shim `containerd-shim-spin`."

2. **Apri i log in tempo reale del Cloud:**
   ```bash
   kubectl logs -l core.spinoperator.dev/app-name=hello-wasm -f
   ```
   *Spiega al professore:* "Ora stiamo osservando i log live del server WebAssembly sul cloud. È in attesa di richieste."

3. **In una nuova scheda, attiva il proxy per esporre il Cloud al nostro Dispositivo Edge virtuale:**
   ```bash
   kubectl port-forward svc/hello-wasm 8080:80
   ```
   *Spiega al professore:* "Questo collega la porta 8080 del nostro PC locale al servizio SpinKube isolato all'interno del cluster Kubernetes."

---

## Step 2: Rete del Dev Container (Il Ponte L2)

**Obiettivo:** Spiegare l'infrastruttura di test che bypassa i limiti dell'emulatore.

1. **Apri un terminale all'interno del Dev Container.**
2. **Avvia il demone Socat (se non l'hai automatizzato):**
   ```bash
   socat TCP-LISTEN:8080,fork TCP:host.docker.internal:8080 &
   ```
   *Spiega al professore:* "Poiché l'emulatore hardware QEMU User Networking è isolato, usiamo un piccolo proxy TCP per fargli credere che il Cloud sia sulla gateway di default fisso `10.0.2.2`. Su una scheda hardware reale col WiFi o Ethernet, questo passaggio sparirebbe totalmente e la scheda contatterebbe direttamente l'IP di Kubernetes."

---

## Step 3: Mostrare l'Edge (Zephyr / WAMR)

**Obiettivo:** Compilare e avviare il firmware del microcontrollore.

1. **Sempre dal terminale del Dev Container (cartella `/workspaces/MasterThesis/workspace`), mostra il codice C:**
   Apri `app/src/main.c` e sottolinea il passaggio in cui il Runtime WebAssembly (WAMR) carica l'Array C (il payload Wasm):
   *Spiega al professore:* "Questo è il codice del microcontrollore (Zephyr). Non stiamo lanciando script di alto livello, stiamo inizializzando l'interprete WAMR e caricando il bytecode WebAssembly statico bruciato in memoria."

2. **Compila ed Esegui il Boot dell'OS Zephyr:**
   ```bash
   west build -p always -b qemu_x86 app/ && west build -t run
   ```

---

## Step 4: Il Momento "Aha!" (La Connessione End-to-End)

Questo è il momento clou della demo:

1. **Guarda il terminale del Dev Container (Zephyr):**
   Osserverai il classico boot bootloader di Zephyr, poi i log di WAMR che entra in Sandbox. 
   Vedrai le stampe provenienti dall'interno del Wasm Rust (es. connessione al socket, invio dati...).
   *Spiega al professore:* "Il modulo Wasm sull'Edge sta interfacciandosi con le API POSIX/WASI del Kernel Zephyr per aprire un Socket TCP in uscita."

2. **Torna immediatamente al terminale dei Log del Cloud (Passaggio 1.2):**
   Dovresti vedere apparire la chiamata HTTP in arrivo (`GET / HTTP/1.1`) originata dalla QEMU board, con la risposta "Hello World".
   *Spiega al professore:* "Il server SpinKube ha appena registrato e risposto alla richiesta. Abbiamo un flusso end-to-end completo scambiato da due binari agnostici Wasm, uno su Server Cloud distribuito, l'altro su Single-Chip ultra vincolato in SRAM."

---

## Step 5: Conclusione e Discussione

Chiudi il test (usando `Ctrl + A, X` per uscire da QEMU) e fai le tue considerazioni finali al prof:

* **Impronta Memoria:** Sottolinea come l'eseguibile Wasm sull'Edge sia minuscolo e limitato a uno stack di pochi KB definiti dal tuo `prj.conf`.
* **Portabilità Hardcore:** Sottolinea che il file `.wasm` sorgente che ha generato il Client TCP sull'Edge, funzionerebbe esattamente identicamente su un'architettura CPU diversa (`ARM Cortex` su una board STM32 fisica) senza ricompilarlo.
* **Sicurezza by Default:** Menziona l'error `EACCES` che ricevevi prima di configurare la "WASI Address Pool". Spiega che Wasm è una Sandbox ad isolation totale: un modulo Wasm compromesso o estraneo NON può fare richieste di rete verso il macro-sistema senza che l'Host C Zephyr lo autorizzi esplicitamente all'avvio nel `main.c`.
