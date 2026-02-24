# Guida alla Disinstallazione e Pulizia Totale (Teardown)

Una volta terminata e discussa la tua tesi magistrale, potresti voler riportare il tuo Mac al suo stato originario, eliminando tutti i tool di sviluppo, i container e le configurazioni "particolari" che abbiamo aggiunto per far funzionare WebAssembly, SpinKube e Zephyr.

Segui questi passaggi nell'ordine indicato per una pulizia completa e sicura.

---

## 1. Pulizia dei Container e del Cluster (Docker & k3d)

Tutto ciò che riguarda Kubernetes (nodi, configurazioni di containerd modificate, pod) vive fortunatamente all'interno di Docker. Distruggendo i cluster k3d cancellerai automaticamente ogni configurazione o demone installato al loro interno.

Apri il terminale ed esegui:

```bash
# 1. Elimina l'intero cluster Kubernetes
k3d cluster delete wasm-cluster

# 2. Elimina il registro locale (se ancora presente)
k3d registry delete wasm-registry

# 3. Elimina il registro di base creato per i test
docker rm -f k3d-wasm-registry k8s-registry

# 4. Pulizia profonda di Docker: 
# Elimina TUTTI i container fermi, le reti non usate, i volumi e TUTTE le immagini 
# scaricate (comprese le gigantesche immagini di Zephyr e SpinKube).
# *Attenzione*: Questo eliminerà tutte le cache di Docker.
docker system prune -a --volumes
```

---

## 2. Pulizia dei Tool di Sviluppo (macOS)

Abbiamo disinstallato alcune librerie predefinite per fare spazio agli strumenti ufficiali di WebAssembly e Rust. Ora li rimuoviamo.

### Rimuovere Fermyon Spin
Abbiamo installato l'eseguibile CLI di Spin tramite Homebrew.

```bash
# Disinstalla Fermyon Spin
brew uninstall fermyon/tap/spin

# Rimuovi la repository di terze parti di Fermyon da Homebrew
brew untap fermyon/tap
```

*(Opzionale)*: Se usavi il "model checker" `spin` precedentemente, puoi reinstallarlo con: `brew install spin`.

### Rimuovere Rust (Rustup)
Per supportare in modo nativo il target `wasm32-wasip1`, abbiamo sostituito il Rust di Homebrew con l'installer ufficiale `rustup`. Se non programmi in Rust quotidianamente o preferivi l'installazione precedente, eliminiamo anche questo:

```bash
# Questo comando elimina completamente ~/.cargo, ~/.rustup e tutte le toolchain
rustup self uninstall
```

*(Opzionale)*: Se prima usavi il Rust base di Homebrew, puoi ripristinarlo con: `brew install rust`.

---

## 3. Pulizia Cloud (GitHub Container Registry)

Per far comunicare Kubernetes con Internet, abbiamo caricato il file `.wasm` sul tuo account GitHub e generato una chiave di sicurezza (PAT). CANCELLIAMO tutto per questioni di sicurezza.

1. **Eliminare il Pacchetto OCI da GitHub:**
   * Vai sul tuo profilo [GitHub Packages](https://github.com/antoniodev0?tab=packages).
   * Clicca sul pacchetto **`hello-wasm`**.
   * Clicca su **Package Settings** (menù a destra).
   * Scorri fino in fondo in **Danger Zone** e clicca **Delete package**. Segui le istruzioni a schermo per confermare.

2. **Revocare il Token di Accesso (PAT):**
   * Vai su GitHub > Settings > **Developer settings** > **Personal access tokens** > **Tokens (classic)**.
   * Trova il token che hai chiamato `SpinKube Push` (o simile).
   * Clicca sul tasto **Delete** accanto al Token per revocarne permanentemente i permessi.

---

## 4. Pulizia dei File di Progetto

Come ultimo passaggio, quando la tesi sarà consolidata sul documento finale, ti basterà prendere interamente la cartella principale del progetto:

```bash
# Vai alla Scrivania o dove si trova il progetto
cd /Users/antonio/Desktop/UNI/Tesi/

# Elimina l'intero progetto (incluso il devcontainer e il codice compilato)
rm -rf MasterThesis
```

E con questo, il tuo Mac sarà esattamente come prima che iniziassimo a costruire la WebAssembly Sandbox! 🧹✨
