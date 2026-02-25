# Integrazione Edge-to-Cloud con WebAssembly (Zephyr OS & SpinKube)

## Introduzione
Questo documento descrive l'architettura, le implementazioni e i problemi risolti durante la realizzazione di una pipeline di comunicazione Edge-to-Cloud basata su WebAssembly (Wasm). Il dispositivo Edge è simulato tramite Zephyr OS (su QEMU) con il WebAssembly Micro Runtime (WAMR), mentre il backend Cloud è ospitato su Kubernetes tramite SpinKube.

## Obiettivo Completato
**l'integrazione via chiamata API (Socket TCP HTTP) funziona correttamente in maniera end-to-end.** Il modulo WebAssembly in esecuzione sul dispositivo Edge (Zephyr) riesce ad instaurare una connessione TCP verso l'applicazione Wasm cloud-native (SpinKube), spedisce una richiesta HTTP GET standard e riceve correttamente la risposta (`Hello World!`).

## Cosa è stato fatto
1. **Sviluppo Modulo Edge (Rust):** È stato sviluppato un modulo Wasm target `wasm32-wasip1` che esegue chiamate di rete tramite API POSIX/WASI verso il backend cloud.
2. **Adattamento Runtime WAMR (Zephyr):** Il firmware Zephyr è stato compilato abilitando il runtime WAMR, le funzionalità POSIX (`CONFIG_POSIX_API`) e lo stack di rete TCP/IPv4.
3. **Deploy Backend Cloud (SpinKube):** È stata creata un'applicazione Spin (HTTP) deployata su un cluster Kubernetes locale (`k3d`), esposta sul computer host tramite port-forwarding.
4. **Networking Dev Container - Host:** È stato configurato un proxy TCP tramite `socat` per instradare il traffico dal gateway di default emulato da QEMU (`10.0.2.2:8080`) verso l'host fisico in cui risiede il cluster K3d.
5. **Integrazione Wasm-to-C FFI:** Sono stati implementati in Rust dei binding FFI sicuri (Foreign Function Interface) verso le chiamate socket `libc-wasi` implementate ad hoc in WAMR (`sock_open`, `sock_connect`, `sock_send`, `sock_recv`).

## Problemi Riscontrati e Soluzioni Proposte

### 1. Incompatibilità delle API WASI Socket Standard
* **Problema:** L'utilizzo di librerie Rust di alto livello (`std::net`) o crate specifici affermati come `wasmedge_wasi_socket` causava fallimenti durante il caricamento o l'esecuzione del modulo Wasm. Il runtime WAMR per le API Socket si discosta dalle signature di altri runtime WebAssembly ed esige tipi di dato custom.
* **Soluzione:** Sono stati bypassati i crate di rete Rust built-in, definendo a mano i blocchi logici ABI (`extern "C"`) in Rust per interfacciarsi con il core WAMR C-based (`wasmtime_ssp_sock_...`), curando attentamente i layout di memoria (es. `#repr(C)`) delle struct.

### 2. Errori di Connessione e Timeout (`EPROTOTYPE`, `EACCES`, `ETIMEDOUT`)
* **Problema (`EPROTOTYPE`):** Inizialmente lo stack TCP in Zephyr risultava disabilitato.
  * **Soluzione:** Inserito il flag `CONFIG_NET_TCP=y` in `prj.conf`.
* **Problema (`EACCES`):** Livello di sicurezza strict di WAMR; impediva al payload il traffico in rete generico verso la subnet dell'host.
  * **Soluzione:** Espanso il "WASI Address Pool" invocando le API C (`wasm_runtime_set_wasi_addr_pool`) in `main.c`, dando i permessi d'uscita alla rete.
* **Problema (`ETIMEDOUT`):** QEMU (in modalità User SLIRP) non riusciva ad instradare pacchetti verso determinati indirizzi di interfacce host Docker (`192.168.65.254`).
  * **Soluzione:** Si è forzato il collegamento della board al "Magic IP" gateway di QEMU User Networking (`10.0.2.2`). Da lì, un proxy in esecuzione nel Dev Container deviava la richiesta all'IP finale sul cluster K3d Kubernetes.

### 3. Loop/Hang del processo di invio dati (`sock_send` e `sock_recv`)
* **Problema:** Nonostante il TCP Handshake (`sock_connect`) completasse l'operazione con successo, la scrittura HTTP payload freezava la scheda.
* **Soluzione:** I buffer di Array Byte I/O richiesti da WAMR sono conformi alle primitive WASI e impongono puntatori specifici ai Vector I/O Arrays (`wasi_ciovec_t`). La re-implementazione custom dei tipi FFI `WasiCiovec` nel modulo Rust per combaciare perfettamente ai requisiti di WAMR ha consentito lo stream completo, risolvendo l'hang.

## Ambito Emulato vs Scenario Hardware Reale

Cosa sarebbe cambiato se non avessimo lavorato virtualizzando una CPU tramite QEMU nel Dev Container, e avessimo invece impiegato hardware reale embedded (es. nRF52, ESP32, STM32, Arduino Nano 33)?

### 1. Topologia di Rete Semplificata
* **Emulato:** La parte di setup rete ha generato colli di bottiglia e un effort notevole a causa del Multi-Layer Networking imposto dalla natura del simulatore: `Applicazione SpinKube` <-> `Port Forwarding Host Locale` <-> `VLAN Docker (Dev Container)` <-> `Scheda QEMU`. 
* **Scenario Reale:** Su una scheda reale, il dispositivo Edge instaura un link L2 (es. WiFi, Ethernet, o LTE MAC) collegandosi direttamente ad un Router e ricevendo un indirizzo IP, diventando "peer" nella rete. Non servirebbero ponti virtuali SLIRP o proxy esterni (`socat`); la board scambierebbe le frame TCP direttamente con l'endpoint di backend esposto in Public IP o Local IP. Il file `prj.conf` diventerebbe più minimale, ignorando driver fantasma (`CONFIG_NET_QEMU_USER` o `CONFIG_ETH_E1000`).

### 2. Architettura CPU e Toolchain Reale
* **Emulato:** Per facilitare i test a rapido loop iterativo abbiamo emulato una build cross-compilata basata su ISA `x86` (`west build -b qemu_x86`).
* **Scenario Reale:** Eseguendo una build custom targatizzata (es. `west build -b nucleo_wb55rg`), si attiverebbe il processo di flashing via USB verso controller ARM Cortex o RISC-V. Un grande vantaggio del WebAssembly emerge proprio qui: il file binario `.wasm` payload generato da Rust per il target `wasm32-wasip1` **rimarrebbe del tutto invariato** e funzionerebbe identicamente in ambiente ARM o RISC-V proprio come ha funzionato su intel/AMD `x86`, dato che WAMR porta con sé l'interprete ISA-agnostic.

### 3. Dispositivi periferici I/O Hardware Reale
* **Scenario Reale:** Nello scenario fisico reale, uno sviluppatore Wasm implementerebbe le interfacce WAMR verso driver Host Hardware reali. Invece di inviare esclusivamente una stringa statica `"GET / HTTP/..."`, il dispositivo fisico esegue polling sui sensori collegati in UART, I2C, SPI o ADC (Digital to Analog) ad esempio captando umidità, radiazioni o temperature — mandando a SpinKube la vera telemetria edge, valorizzando così il modulo Wasm compilato. Le limitazioni intrinseche della RAM/SRAM sulla board avrebbero richiesto l'allocazione più conservativa del footprint di WAMR rispetto all'emulazione dove avevamo MegaBytes di agio. L'ambiente embedded hardware reale esige ottimizzazioni AOT (Ahead-Of-Time compile) supportato da WAMR per limare il codice.
