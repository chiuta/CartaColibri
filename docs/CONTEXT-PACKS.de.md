# Kontextpakete

Mit der eigenen Engine von Colibri Companion (`corbul serve --packs DIR`) kann CartaColibri die Arbeit an jeder Quelle über Aufrufe hinweg wiederverwenden: erst zusammenfassen, dann extrahieren, dann vergleichen, über dieselben Dokumente, ohne sie erneut zu lesen.

- Einschalten mit `--packs` (jeder Befehl mit `--url`) oder `COLI_PACKS=1`.
- Jede Quelle eines Prompts wird zu einem Paket, überschrieben mit ihrem Namen und einem kurzen #Tag statt ihres [S#]-Labels (das Label hängt von der Position ab, derselbe Text würde sich also von Aufruf zu Aufruf unterscheiden). Das Ende des Prompts ordnet jedem #Tag sein [S#]-Label zu; Zitate und ihre Prüfung funktionieren wie bisher.
- Jede Engine ohne `/v1/packs/complete` (colibri, Ollama, Cloud-Dienste) wird einmal erkannt, dann läuft der normale Chat. Cloud-Backends mit `--provider` bekommen nie Pakete.
- Die Antwort ist eine reine Textvervollständigung ohne Chat-Vorlage: kleine instruktionsgetunte Modelle folgen den Anweisungen dort womöglich schlechter als im Chat. Mit `--verify` prüfen.
- Auf echten Modellen noch nicht gemessen.

Das Regal lässt sich halb so groß und komprimiert halten, wenn corbul mit `--packs-kv f16` und `--packs-compress` gestartet wird (F16-Rundung in jeder Zeile; die Kompression ist verlustfrei).

Passen die Quellen nicht in das Fenster des Modells, lässt `--packs-shrink` (oder `COLI_PACKS_SHRINK=1`) corbul sie auf die wichtigsten Zeilen verkleinern, statt zu scheitern. Dabei geht Information verloren; siehe `docs/SHRINK-MAP.de.md` von Colibri Companion.
