# Pachete de context

Cu motorul propriu al lui Colibri Companion (`corbul serve --packs DIR`), CartaColibri poate refolosi munca făcută pe fiecare sursă între apeluri: rezumat, apoi extragere, apoi comparare pe aceleași documente, fără a le citi din nou.

- Pornește cu `--packs` (orice comandă care primește `--url`) sau `COLI_PACKS=1`.
- Fiecare sursă dintr-un prompt devine un pachet cu numele și un #tag scurt în loc de eticheta [S#] (eticheta depinde de poziție, deci același text ar diferi de la un apel la altul). Coada promptului leagă fiecare #tag de eticheta [S#], deci citările și verificarea lor merg ca până acum.
- Orice motor fără `/v1/packs/complete` (colibri, Ollama, servicii cloud) e detectat o dată și se folosește chatul obișnuit. Backend-urile cloud `--provider` nu primesc niciodată pachete.
- Răspunsul e o completare de text simplu, fără șablonul de chat al modelului: modelele mici ajustate pe instrucțiuni pot urma instrucțiunile mai slab decât în chat. Verifică cu `--verify`.
- Nemăsurat încă pe modele reale.
