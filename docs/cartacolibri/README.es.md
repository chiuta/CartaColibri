<p align="center"><img src="../../assets/cartacolibri-logo.png" width="320" alt="CartaColibri — Local AI Library"></p>

<p align="center"><b>Tus documentos, buscables y explicados — 100 % en local.</b></p>

<p align="center">[English](README.en.md) · [Română](README.ro.md) · [Français](README.fr.md) · [Italiano](README.it.md) · <b>Español</b> · [Português](README.pt.md) · [Deutsch](README.de.md) · [Magyar](README.hu.md)</p>

CartaColibri es una biblioteca de IA local construida sobre el motor Colibrì. Indexa las carpetas que eliges, te deja buscar en ellas al instante y luego pide a un modelo de lenguaje local que resuma, extraiga datos o sintetice grupos de archivos, citando siempre el archivo de origen. Nada sale de tu ordenador y tus originales nunca se modifican.

## Qué hace

- **Biblioteca** — todos los documentos locales con su ubicación; añade o quita carpetas, sube archivos, míralos y, si quieres, guarda el texto OCR.
- **RAG-fetch** — índice de texto completo incremental al estilo DocFetcher (palabras, «frases», prefijo*, AND/OR/NOT, filtros por tipo, tamaño y ubicación) y **grupos** de archivos reutilizables.
- **RAG-CARTA** — tres tareas locales sobre un grupo: **resumir**, **extraer** (tabla o JSON, «NOT FOUND» en lugar de suposiciones) y **sintetizar** (temas, puntos comunes, contradicciones). Cada respuesta cita [S1], [S2]… y enumera sus fuentes.
- **Privada por diseño** — sin nube ni telemetría; los originales solo se leen. 8 idiomas: EN, RO, FR, IT, ES, PT, DE, HU.
- **web-grab** — guarda cualquier página web (o solo tu selección) en la biblioteca como texto limpio, conservando su dirección como fuente; un comando, un bookmarklet que funciona en cualquier navegador (Falkon incluido) o una plantilla de plugin para Falkon.

## Inicio rápido

Compílala (o descarga el zip portátil) y arranca la página local. Para las tareas de IA, arranca también el motor Colibrì con un modelo que quepa en tu RAM (mira su documentación):

```bash
git clone https://github.com/chiuta/CartaColibri
cd CartaColibri
cargo build --release                       # Rust 1.90+, or use the portable zip
./target/release/cartacolibri serve         # then open http://127.0.0.1:8765/
coli web --model ~/colibri-models/<model>   # Colibrì engine, only for the AI tasks
```

O usa la línea de comandos, sin modelo para probar (`--mock` es un modo sin conexión):

```bash
cartacolibri add ~/Documents
cartacolibri search "contract*" --ext pdf,docx
cartacolibri group set Contracts ~/Documents/contracts
cartacolibri synthesize "payment terms" --group Contracts --save --mock
cartacolibri grab https://example.org/article      # web-grab
```

## Cómo encaja todo

Capa 0 — tus carpetas (solo lectura) → RAG-fetch — índice, búsqueda, grupos → RAG-CARTA — tareas de IA con el motor Colibrì.

## Licencia y créditos

CartaColibri se publica bajo **licencia MIT**. Es una implementación Rust clean-room: habla con el motor [Colibrì](https://github.com/JustVugg/colibri) (JustVugg, Apache-2.0) solo a través de su pasarela HTTP pública y no contiene nada de su código. La capa de índice y búsqueda sigue el comportamiento descrito en el manual público de DocFetcher; no se usa código de DocFetcher. La plantilla de plugin para Falkon, opcional, es un archivo aparte que solo habla con CartaColibri por HTTP local.

## Más información

[Documentación completa](../cartacolibri.md) · [README del motor Colibrì](https://github.com/JustVugg/colibri) · [Contribuir](../../CONTRIBUTING.md) · [📄 Tutorial PDF ilustrado](CartaColibri-Tutorial.es.pdf)
