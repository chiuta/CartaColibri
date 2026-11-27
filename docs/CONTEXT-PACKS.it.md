# Pacchetti di contesto

Con il motore proprio di Colibri Companion (`corbul serve --packs DIR`), CartaColibri può riusare il lavoro fatto su ogni fonte tra una chiamata e l'altra: riassumere, poi estrarre, poi confrontare sugli stessi documenti senza rileggerli.

- Si attiva con `--packs` (ogni comando che accetta `--url`) o `COLI_PACKS=1`.
- Ogni fonte di un prompt diventa un pacchetto intitolato dal nome e da un breve #tag al posto dell'etichetta [S#] (l'etichetta dipende dalla posizione, quindi lo stesso testo cambierebbe da una chiamata all'altra). La coda del prompt collega ogni #tag alla sua etichetta [S#]: le citazioni e i loro controlli funzionano come prima.
- Ogni motore senza `/v1/packs/complete` (colibri, Ollama, servizi cloud) viene riconosciuto una volta e si usa la chat normale. I backend cloud `--provider` non ricevono mai pacchetti.
- La risposta è un completamento di testo semplice, senza il template di chat: i modelli piccoli addestrati sulle istruzioni possono seguirle peggio che in chat. Controlla con `--verify`.
- Non ancora misurato su modelli reali.
