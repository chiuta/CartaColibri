# Kontextcsomagok

A Colibri Companion saját motorjával (`corbul serve --packs DIR`) a CartaColibri hívások között újrahasznosíthatja az egyes forrásokon végzett munkát: összefoglalás, majd kinyerés, majd összehasonlítás ugyanazokon a dokumentumokon, újraolvasás nélkül.

- Bekapcsolás: `--packs` (bármely `--url`-t fogadó parancs) vagy `COLI_PACKS=1`.
- A prompt minden forrása olyan csomag lesz, amelynek fejléce a neve és egy rövid #címke a [S#] címke helyett (a címke a pozíciótól függ, így ugyanaz a szöveg hívásról hívásra eltérne). A prompt vége minden #címkét összekapcsol a [S#] címkéjével, így az idézetek és ellenőrzésük a megszokott módon működik.
- Minden `/v1/packs/complete` nélküli motort (colibri, Ollama, felhőszolgáltatások) egyszer észlel, utána a szokásos csevegés fut. A felhős `--provider` háttérrendszerek soha nem kapnak csomagot.
- A válasz sima szövegkiegészítés csevegősablon nélkül: a kis, utasításra hangolt modellek rosszabbul követhetik az utasításokat, mint csevegésben. Ellenőrzés: `--verify`.
- Valódi modelleken még nem mértük.

A polc fél méretben és tömörítve tartható, ha a corbult a `--packs-kv f16` és `--packs-compress` kapcsolóval indítod (F16-kerekítés minden soron; a tömörítés veszteségmentes).

Ha a források nem férnek a modell ablakába, a `--packs-shrink` (vagy `COLI_PACKS_SHRINK=1`) engedi, hogy a corbul a legfontosabb sorokra zsugorítsa őket a hibázás helyett. Információ vész el; lásd a Colibri Companion `docs/SHRINK-MAP.hu.md` fájlját.
