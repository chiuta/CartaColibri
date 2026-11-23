<p align="center"><img src="../../assets/cartacolibri-logo.png" width="320" alt="CartaColibri — Local AI Library"></p>

<p align="center"><b>Tus documentos, buscables y explicados — 100 % en local.</b></p>

<p align="center">[English](README.en.md) · [Română](README.ro.md) · [Français](README.fr.md) · [Italiano](README.it.md) · <b>Español</b> · [Português](README.pt.md) · [Deutsch](README.de.md) · [Magyar](README.hu.md)</p>

CartaColibri es una biblioteca de IA local construida sobre el motor Colibrì. Indexa las carpetas que eliges, te deja buscar en ellas al instante y luego pide a un modelo de lenguaje local que resuma, extraiga datos o sintetice grupos de archivos, citando siempre el archivo de origen. Nada sale de tu ordenador y tus originales nunca se modifican.

## Qué hace

- **Biblioteca** — todos los documentos locales con su ubicación; añade o quita carpetas, sube archivos, míralos y, si quieres, guarda el texto OCR.
- **RAG-fetch** — índice de texto completo incremental al estilo DocFetcher (palabras, «frases», prefijo*, AND/OR/NOT, filtros por tipo, tamaño y ubicación) y **grupos** de archivos reutilizables.
- **RAG-CARTA** — tres tareas locales sobre un grupo: **resumir**, **extraer** (tabla o JSON, «NOT FOUND» en lugar de suposiciones) y **sintetizar** (temas, puntos comunes, contradicciones). Cada respuesta cita [S1], [S2]… y enumera sus fuentes.
- **Privada por diseño** — sin nube ni telemetría; los originales solo se leen. 8 idiomas: EN, RO, FR, IT, ES, PT, DE, HU.

## Inicio rápido

Descarga el código y arranca el motor con un modelo que quepa en tu RAM (mira la documentación de Colibrì); después abre la página de CartaColibri:

```bash
git clone https://github.com/chiuta/CartaColibri
cd CartaColibri
./start-here.sh            # Windows: START-HERE.bat
coli web --model ~/colibri-models/<model>   # then open http://127.0.0.1:8790/carta.html
```

O usa la línea de comandos, sin modelo para probar (`--mock` es un modo sin conexión):

```bash
coli carta add ~/Documents
coli carta search "contract*" --ext pdf,docx
coli carta group set Contracts ~/Documents/contracts
coli carta synthesize "payment terms" --group Contracts --save --mock
```

## Cómo encaja todo

Capa 0 — tus carpetas (solo lectura) → RAG-fetch — índice, búsqueda, grupos → RAG-CARTA — tareas de IA con el motor Colibrì.

## Licencia y créditos

CartaColibri es un fork de [Colibrì](https://github.com/JustVugg/colibri) de JustVugg y mantiene su licencia Apache-2.0. La capa de índice y búsqueda es un diseño clean-room inspirado en el comportamiento descrito en el manual público de DocFetcher; no se usa código de DocFetcher.

## Más información

[Documentación completa](../cartacolibri.md) · [README del motor Colibrì](../../README.colibri.md) · [Contribuir](../../CONTRIBUTING-CARTA.md)
