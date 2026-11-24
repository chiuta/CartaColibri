<p align="center"><img src="../../assets/cartacolibri-logo.png" width="320" alt="CartaColibri — Local AI Library"></p>

<p align="center"><b>Tus documentos, buscables y explicados — 100 % en local.</b></p>

<p align="center">[English](README.en.md) · [Română](README.ro.md) · [Français](README.fr.md) · [Italiano](README.it.md) · <b>Español</b> · [Português](README.pt.md) · [Deutsch](README.de.md) · [Magyar](README.hu.md)</p>

CartaColibri es una biblioteca de IA local construida sobre el motor Colibrì. Indexa las carpetas que eliges, te deja buscar en ellas al instante y luego pide a un modelo de lenguaje local que resuma, extraiga datos o sintetice grupos de archivos, citando siempre el archivo de origen. Nada sale de tu ordenador y tus originales nunca se modifican.

## Tus primeros 5 minutos

No hace falta experiencia. Tres pasos y habrás buscado y resumido tus propios archivos.

1. **Arráncala.** Descomprime el zip portátil y ejecuta el lanzador, o escribe `cartacolibri start` en una terminal. Tu navegador muestra la página local, `http://127.0.0.1:8765/`.
2. **Añade una carpeta.** En la página abre **Biblioteca**, escribe la ruta de una carpeta como `~/Documents` y haz clic en **Añadir carpeta**. La primera indexación de una carpeta grande puede tardar unos minutos. Tus archivos solo se leen, nunca se modifican.
3. **Busca y luego pregunta.** Abre **RAG-fetch**, busca una palabra, marca unos cuantos archivos y crea un grupo. Después abre **RAG-CARTA**, elige **Resumir** y haz clic en **Ejecutar**. ¿Aún no tienes un modelo de IA? Marca **Modo sin conexión (sin modelo)** y pruébalo igualmente.

¿Prefieres la terminal? Los mismos tres pasos:

```bash
cartacolibri start                 # step 1 - leave this window open (use a second one for the rest)
cartacolibri add ~/Documents       # step 2
cartacolibri group set Demo ~/Documents
cartacolibri summarize "main points" --group Demo --mock   # step 3 - --mock = no AI model needed
```

## Qué hace

- **Biblioteca** — todos los documentos locales con su ubicación; añade o quita carpetas, sube archivos, míralos y, si quieres, guarda el texto OCR.
- **RAG-fetch** — índice de texto completo incremental al estilo DocFetcher (palabras, «frases», prefijo*, AND/OR/NOT, filtros por tipo, tamaño y ubicación) y **grupos** de archivos reutilizables.
- **RAG-CARTA** — tres tareas locales sobre un grupo: **resumir**, **extraer** (tabla o JSON, «NOT FOUND» en lugar de suposiciones) y **sintetizar** (temas, puntos comunes, contradicciones). Cada respuesta cita [S1], [S2]… y enumera sus fuentes.
- **Privada por diseño** — sin nube ni telemetría; los originales solo se leen. 8 idiomas: EN, RO, FR, IT, ES, PT, DE, HU.
- **web-grab** — un pequeño navegador en Rust (clean-room) con pestañas, que conoce tu biblioteca: una insignia cuando la página ya está guardada (y cuando ha cambiado), documentos relacionados mientras navegas, guardado con un clic, preguntas a tu Colibrì local sobre una página — o sobre todas las pestañas abiertas, con citas. Un comando y un bookmarklet cubren otros navegadores. Además: 🖍 resaltados guardados con la página, 👁 páginas vigiladas con el detalle de los cambios (`cartacolibri webwatch`), instantánea HTML en cada guardado, historial con búsqueda opcional (`cartacolibri history on`), 🗂📝📊 resumen o extracción entre pestañas y `?pregunta` en la barra de direcciones para consultar toda tu biblioteca.

## Funciones avanzadas

- **Encuentra** — una barra de progreso durante la indexación; «¿Querías decir…?» para los errores de escritura, con sugerencias cuando nada coincide; los duplicados listados y un motivo claro para cada archivo omitido.
- **Organiza** — etiquetas, una estrella y una nota privada en cada documento; sobreviven a la reindexación. Mira los documentos relacionados, lo que abriste hace poco y con más frecuencia, y guarda búsquedas como colecciones inteligentes.
- **14 tareas** — comparar, cronología, glosario, esquema, acciones, FAQ, tarjetas, cuestionario, traducir, explicar, contradicciones, además de resumir, extraer y sintetizar. Marca **verificar citas** para señalar las frases que las fuentes no respaldan y elige el idioma de la respuesta.
- **Chat** — haz preguntas encadenadas sobre toda tu biblioteca o sobre un grupo. Las respuestas citan sus fuentes o dicen con honestidad «no se encontró en tus documentos» en vez de adivinar.
- **Comparte y protege** — exporta un grupo, una etiqueta o una búsqueda como Markdown, CSV, JSON o zip, con citas; una vista de Estadísticas con gráficos y cronología; una copia de seguridad en un solo archivo con sumas de verificación; Comprobar/Reparar para el índice.
- **Trae cosas** — importa los marcadores del navegador, una carpeta de Obsidian, Zotero/BibTeX, CSV de Pocket/Raindrop o exportaciones de Evernote, y sigue fuentes RSS/Atom (OPML de entrada y salida): se convierten en notas con búsqueda. También se indexan correos, calendarios, subtítulos y cuadernos.

## Novedades de la 0.2.0-alpha.4

- **Filtros directamente en la casilla de búsqueda** — Escribe `ext:pdf`, `path:contratos` o `size:>1mb` junto a tus palabras y los resultados se acotan. Funcionan en la página web, en la línea de comandos y a través de la API.
- **Búsqueda inteligente** — Marca «Búsqueda inteligente» y tu IA local añade algunos sinónimos y palabras afines (como un simple OR: las coincidencias exactas siguen primero). Las palabras añadidas se muestran bajo la casilla. Sin motor en marcha obtienes la búsqueda normal.
- **Chat de pregunta única** — Marca «Cada pregunta por separado» y el chat olvida las preguntas anteriores: cada respuesta sale solo de tus documentos, con sus fuentes.
- **Un servidor MCP para otras apps de IA** — `cartacolibri mcp` permite que otras aplicaciones de IA busquen en tu biblioteca, lean un documento y le hagan preguntas mediante el Model Context Protocol. Es de solo lectura, se ejecuta en tu ordenador y solo llega a los documentos que devolvió la búsqueda.

## Novedades de la 0.2.0-alpha.2

- **La dirección del motor funciona sola** — `http://127.0.0.1:8000` y `http://127.0.0.1:8000/v1` valen las dos. CartaColibri lee la lista de modelos en `/v1/models`, elige solo el modelo cargado y ya no necesita una página `/health`. Esto arregla el mensaje «AI engine not running (404)».
- **Encuentra tu IA solo** — CartaColibri busca un motor en marcha en este ordenador: los ajustes propios de Colibri Companion, un archivo de descubrimiento y después los puertos locales habituales (8000, 8080, 11434, 1234, 8001, 5000). Solo se sondean direcciones loopback. Abre ⚙ para ver lo que encontró o pulsa «Iniciar Colibri Companion».
- **Estado y velocidad en directo** — La cabecera muestra el modelo en marcha y si se encontró automáticamente, actualizado cada pocos segundos. Bajo cada respuesta ves los tokens usados y la velocidad; por debajo de 3 tokens/s recibes el consejo de elegir un modelo más pequeño. Las peticiones llegan al motor de una en una.
- **Páginas web desde el diálogo de importación** — En «Traer cosas de otro sitio» los canales de noticias dieron paso a web-grab: pega una dirección y guarda la página como un único archivo HTML o MHTML en tu biblioteca. Los canales siguen ahí, plegados.
- **Tres botones en el navegador web-grab** — «Save to folder» guarda la copia de texto en tu biblioteca, «Grab website HTML» guarda la página (Mayús+clic para MHTML), «PDF» la imprime en PDF. El PDF es silencioso en Linux; en Windows y macOS se abre el diálogo de impresión y eliges «Guardar como PDF».
- **Las páginas guardadas se pueden buscar** — Los archivos `.mhtml` y `.html` se indexan con el juego de caracteres correcto, así que una página guardada se encuentra como cualquier documento. Las partes de una página que viven en direcciones privadas nunca se descargan ni se enlazan.

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
cartacolibri grab https://example.org/article      # web-grab (command line)
cartacolibri-browser https://example.org           # web-grab (browser)
```

## Más comandos

| Comando | Qué hace |
|---|---|
| `task NAME [request] --group G \| --tag T \| --query Q [--mock] [--verify] [--lang xx] [--save md\|html\|txt]` | las 14 tareas (compare, timeline, glossary, outline, actions, faq, flashcards, quiz, translate, explain, contradictions, summarize, extract, synthesize); `--verify` marca las frases cuyo `[S#]` el texto no respalda |
| `chat [--group G]` | preguntas encadenadas sobre tu biblioteca; respuestas citadas o «no se encontró en tus documentos» |
| `tag add/rm/list/show`, `star`, `note` | tu organización personal, que se conserva tras reindexar |
| `saved add/list/run/rm` | búsquedas guardadas / colecciones inteligentes |
| `recent`, `related PATH` | lo que abriste hace poco; documentos parecidos |
| `export --format md\|csv\|json\|zip` | comparte un grupo, una etiqueta o una búsqueda guardada, con citas |
| `backup OUT [--index]`, `backup-check ZIP`, `restore ZIP` | copia de seguridad en un solo archivo con sumas de verificación; compruébala; restáurala |
| `verify [--repair]` | comprueba el índice; con `--repair` lo reconstruye a partir de tus archivos |
| `stats`, `timeline` | cifras de tu biblioteca; documentos por mes |
| `import PATH` | marcadores `.html`, carpeta de Obsidian, `.bib`, Pocket/Raindrop `.csv`, Evernote `.enex` |
| `feeds add/list/rm/poll`, `feeds opml-import/opml-export` | suscripciones RSS/Atom guardadas como notas |
| `webwatch`, `history` | páginas web vigiladas con diferencias; historial de navegación opcional |

## Cómo encaja todo

Capa 0 — tus carpetas (solo lectura) → RAG-fetch — índice, búsqueda, grupos → RAG-CARTA — tareas de IA con el motor Colibrì.

## Preguntas frecuentes

**¿Qué significa «local»?**  
Todo se ejecuta en tu propio ordenador: la página, el índice de búsqueda y el modelo de IA. Funciona incluso sin red. La única excepción es web-grab, que descarga las páginas web que le pides.

**¿Se envían mis datos a algún sitio?**  
No. No hay nube, ni cuenta, ni telemetría, ni analítica de uso. El único tráfico que sale de tu equipo son las páginas web que pides explícitamente a web-grab que descargue. Detalles en «Privacidad y seguridad».

**Colibrì no está en marcha. ¿Puedo usarla igualmente?**  
Sí. La Biblioteca y la búsqueda nunca necesitan un modelo. Para resumir, extraer y sintetizar marca **Modo sin conexión (sin modelo)** (`--mock` en la línea de comandos): obtienes una respuesta sencilla construida con tus archivos, con citas, pero sin redacción real de IA. Para respuestas de verdad, arranca el motor Colibrì.

**¿Qué es Colibrì y lo necesito?**  
[Colibrì](https://github.com/JustVugg/colibri) es un motor local aparte que ejecuta el modelo de lenguaje. Solo lo necesitas para las respuestas de IA. Elige un modelo cuyo archivo sea menor que tu RAM libre y empieza por uno pequeño. Mira la documentación de Colibrì.

**¿Qué archivos puede leer?**  
Texto y notas (txt, md, rst, log, csv, json, yaml, html, rtf), PDF, archivos de Word, OpenDocument, PowerPoint y Excel (docx, odt, pptx, xlsx), epub y, con el OCR activado, imágenes y PDF escaneados.

**¿Y los escaneos y fotos de documentos?**  
Activa el OCR (`--ocr` o `CARTACOLIBRI_OCR=1`). Necesita `tesseract` instalado en tu ordenador, y `pdftoppm` para los PDF escaneados. La página te avisa cuando falta tesseract.

**¿Cómo actualizo?**  
Versión portátil: descarga el zip más reciente y descomprímelo. Desde el código fuente: `git pull` y luego `cargo build --release`. Tus datos en `~/.cartacolibri` se quedan donde están y el índice se refresca solo, únicamente en lo que cambió.

**¿Cómo lo borro todo?**  
Borra la carpeta del programa y después la carpeta de estado `~/.cartacolibri` (o la que indique `CARTACOLIBRI_HOME`). Eso es todo: tus documentos nunca se tocaron.

**¿Algo especial en Windows o macOS?**  
El zip portátil existe para ambos, pero Linux es el sistema mejor probado. El navegador web-grab se compila en CI para Windows y macOS y allí nunca se ha ejecutado a mano. En Windows necesita el runtime de WebView2. Si tu sistema te pide confirmar la apertura de un programa descargado, confirma solo si lo obtuviste de la página del proyecto.

**¿Por qué «alfa»?**  
Porque es joven: el núcleo funciona y está probado, pero no en todos los sistemas, y las comprobaciones automáticas de GitHub aún no están verificadas. Espera algunas asperezas y cuéntanoslas.

**¿Cómo sobreviven las etiquetas, estrellas y notas a la reindexación?**  
No se guardan en el índice de búsqueda, sino en un pequeño archivo aparte, `organize.json`, en tu carpeta `<home>`, ligado a la ruta de cada documento. Así que **Actualizar índice**, una reindexación completa e incluso **Comprobar/Reparar** (que reconstruye el índice) no las tocan. Si renombras o mueves un archivo, cuenta como un documento nuevo y las etiquetas se quedan con la ruta antigua.

**¿Cómo hago una copia de seguridad y la restauro?**  
Abre **Estadísticas** y pulsa **Descargar copia de seguridad** (marca la casilla para incluir el índice), o ejecuta `cartacolibri backup my.zip`. Es un único zip con sumas de verificación que contiene tus etiquetas, notas, grupos, búsquedas guardadas y notas importadas, nunca tus documentos originales. `cartacolibri backup-check my.zip` la comprueba y `cartacolibri restore my.zip` lo devuelve todo a su sitio. Guarda el zip en otro sitio que no sea este ordenador.

**¿Cómo importo marcadores, Obsidian o Zotero?**  
Haz clic en **Traer cosas** en la Biblioteca, o ejecuta `cartacolibri import RUTA`. Entiende una exportación de marcadores del navegador (`.html`), una carpeta de Obsidian, un archivo Zotero/BibTeX `.bib`, archivos `.csv` de Pocket o Raindrop y exportaciones de Evernote `.enex`. Cada elemento se convierte en una pequeña nota de tu biblioteca que puedes buscar, etiquetar y citar. Tus originales solo se leen.

**¿Cómo funcionan las fuentes (feeds)?**  
En **Traer cosas → Fuentes**, pega una dirección RSS o Atom y pulsa **Añadir**. Cuando pulsas **Comprobar ahora** (o ejecutas `cartacolibri feeds poll`), las novedades se guardan como notas y se indexan, así que puedes buscarlas y preguntar sobre ellas. Nada se ejecuta solo en segundo plano. Puedes traer y llevar tus suscripciones como OPML.

**¿Qué hace «verificar citas»?**  
Cuando la IA responde, CartaColibri relee cada frase. Si una frase no tiene cita `[S#]`, o la fuente citada no contiene en realidad lo que dice la frase, se marca como `[unsupported]`. Es una red de seguridad rápida, no una garantía: abre la fuente para todo lo importante.

**¿Por qué se rechazó una página web?**  
web-grab solo descarga páginas públicas. Las direcciones de tu propia máquina o de una red privada (`localhost`, `192.168.x.x`, `10.x.x.x` y similares) se rechazan, para que ninguna página web pueda engañarlo y hacerle leer tus servicios internos. Si de verdad lo quieres, por ejemplo para tu propia intranet, arranca CartaColibri con `CARTACOLIBRI_GRAB_LOCAL=1`.

## Cuando algo falla

Prueba primero `cartacolibri doctor`: revisa tu instalación y te dice qué arreglar.

| Síntoma | Causa probable | Qué hacer |
|---|---|---|
| `cartacolibri: command not found` | La carpeta del programa no está en tu PATH. | Ejecútalo con su ruta completa, por ejemplo `./target/release/cartacolibri`, o usa el lanzador del zip portátil. |
| La página no se abre en `http://127.0.0.1:8765/` | La página local no está en marcha, o el puerto está ocupado. | Ejecuta `cartacolibri start`. Si el puerto está ocupado, ejecuta `cartacolibri serve --port 8766` y abre esa dirección. |
| No sabes qué falla | Falta algo en la configuración. | Ejecuta `cartacolibri doctor`: revisa la configuración y te dice qué arreglar. |
| Las tareas de IA dicen que no alcanzan Colibrì | El motor Colibrì no está en marcha, o escucha en otra dirección (por defecto `http://127.0.0.1:8000`). | Arráncalo (`coli web --model ~/colibri-models/<model>`) o pásale su dirección con `--url`. Mientras tanto, marca **Modo sin conexión (sin modelo)**. |
| Los archivos nuevos no aparecen en la búsqueda | El índice aún no los ha visto. | Haz clic en **Actualizar índice** o ejecuta `cartacolibri index`. `cartacolibri watch` lo mantiene al día automáticamente. |
| Algunos archivos aparecen en «Archivos omitidos» | Están dañados, protegidos, no admitidos o son escaneos. | Ejecuta `cartacolibri problems` para ver el motivo. Para escaneos, activa el OCR. |
| Los escaneos e imágenes no dan texto | El OCR está desactivado o `tesseract` no está instalado. | Instala `tesseract` (y `pdftoppm` para PDF) y añade `--ocr`, o define `CARTACOLIBRI_OCR=1`. |
| La respuesta dice NOT FOUND | Los archivos elegidos no contienen la información. Es a propósito: nunca adivina. | Comprueba que está elegido el grupo correcto, reformula la pregunta o añade al grupo los archivos que faltan. |
| El navegador web-grab no se abre | Falta un componente del sistema. | Linux: instala `libwebkit2gtk-4.1`. Windows: instala el runtime de WebView2. Mientras tanto, `cartacolibri grab URL` funciona sin él. |
| Tu propio script recibe `403 missing X-Colibri-UI header` | Es una comprobación de seguridad, no un fallo. | Envía la cabecera `X-Colibri-UI: 1` en cada llamada a `/carta/*`. |
| El índice parece roto: resultados raros, archivos que faltan, un error sobre la base de datos | El archivo del índice se dañó (por ejemplo por un fallo o un disco lleno) o se desincronizó de tus carpetas. | Abre **Estadísticas → Comprobar / Reparar**, o ejecuta `cartacolibri verify --repair`. Reconstruye el índice a partir de tus archivos; tus etiquetas, notas y grupos se conservan. |
| Se omitió un archivo por ser «demasiado lento» | Leerlo superó el límite de tiempo (120 segundos por defecto), normalmente un PDF enorme o dañado, o un escaneo muy grande. | Sube el límite, por ejemplo `CARTACOLIBRI_EXTRACT_TIMEOUT=600`, y luego **Actualizar índice**. O divide / vuelve a exportar el archivo. El motivo aparece en «Archivos omitidos». |
| «AI engine not running … (404)» | La dirección apunta a algo que no es el motor, o no hay ningún modelo cargado. | Abre ⚙, pulsa «Buscar el motor» y elige la dirección que aparece; valen `…:8000` y `…:8000/v1`. Comprueba que haya un modelo cargado en Colibri Companion. |

## Glosario en palabras sencillas

- **Índice** — Un catálogo de las palabras de tus archivos, guardado para que una búsqueda tarde un parpadeo en vez de releerlo todo.
- **RAG** — «Retrieval-augmented generation»: primero se encuentran los pasajes relevantes en tus archivos y luego la IA responde usando solo esos.
- **Cita [S1]** — Una marca como [S1] o [S2] en una respuesta. Apunta al archivo de origen de esa frase, para que puedas comprobarlo.
- **Grupo** — Una lista con nombre de archivos (un proyecto, un tema) en la que puedes buscar o preguntar una y otra vez.
- **Gateway (pasarela)** — La pequeña dirección web (por defecto `http://127.0.0.1:8000`) por la que CartaColibri habla con el motor Colibrì.
- **Instantánea (snapshot)** — Una copia HTML guardada de una página web, hecha cada vez que la guardas con web-grab, para conservar lo que decía ese día.
- **Watch (vigilancia)** — Dos cosas: `cartacolibri watch` mantiene tu índice al día, y `cartacolibri webwatch` vuelve a descargar las páginas que sigues y muestra qué cambió.

## Privacidad y seguridad

- **Dónde se guarda cada cosa.** Todo lo que escribe CartaColibri vive en una sola carpeta, `<home>`: por defecto `~/.cartacolibri`, o la carpeta de `CARTACOLIBRI_HOME`. Dentro: `index.sqlite` (el índice de búsqueda), `groups.json`, `library/uploads/` (los archivos que subes), `outputs/` (los resultados que elegiste guardar), `snapshots/` y `web.json` (las páginas web guardadas), `highlights.json`, `watch.json`, `settings.json` y `clip.token` (un secreto para el bookmarklet: no lo compartas).
- **Tus originales son de solo lectura.** CartaColibri lee tus documentos y nunca los modifica, mueve ni borra.
- **La página es solo local.** El servidor escucha únicamente en `127.0.0.1`, así que otros ordenadores de tu red no pueden alcanzarlo. También rechaza las peticiones cuya dirección u origen no sea local.
- **La cabecera `X-Colibri-UI`.** Cada llamada de la página a su propio servidor lleva esta cabecera. Una página web que visites en otra pestaña no puede añadirla, así que no puede hacer que tu navegador toque la aplicación local a tus espaldas.
- **Nada sale de `127.0.0.1`, salvo las páginas que pides descargar.** Sin telemetría, sin analítica, sin nube. web-grab solo descarga las direcciones que le das (y las páginas que elegiste vigilar). El sitio de presentación tampoco carga nada de otros servidores.
- **El historial viene desactivado.** El historial del navegador web-grab solo existe si lo activas (`cartacolibri history on`), se queda en tu disco, y `cartacolibri history clear` lo borra.

## Estado real

CartaColibri es software **alfa** (versión 0.2.0-alpha): funciona, pero espera algunas asperezas y guarda copias de seguridad de lo importante. Linux es la plataforma mejor probada: allí el navegador web-grab se autoprueba en CI. En **Windows y macOS** el navegador se compila en CI, pero **nunca se ha ejecutado a mano**. Las comprobaciones automáticas de GitHub (CI) pasan en Linux, Windows y macOS. Tus documentos están a salvo en cualquier caso, porque solo se leen. Las descargas portátiles para Linux, Windows y macOS están en la página **Releases** de GitHub (marcadas como versión preliminar).

## Seguridad

¿Has encontrado una vulnerabilidad? Infórmala en privado, como se describe en [SECURITY.md](../../SECURITY.md); por favor, no abras antes una incidencia pública. Cómo se defiende el servidor (solo loopback, cabecera `X-Colibri-UI`, sin direcciones privadas en web-grab, originales de solo lectura) está descrito en [docs/threat-model.md](../threat-model.md).

## Licencia y créditos

CartaColibri se publica bajo **licencia MIT**. Es una implementación Rust clean-room: habla con el motor [Colibrì](https://github.com/JustVugg/colibri) (JustVugg, Apache-2.0) solo a través de su pasarela HTTP pública y no contiene nada de su código. La capa de índice y búsqueda sigue el comportamiento descrito en el manual público de DocFetcher; no se usa código de DocFetcher. El navegador web-grab usa el webview del propio sistema operativo y no contiene código de Falkon.

**Alexandru-Ionuț Chiuță** · **Alexandru Dumbrava** (coautor)

## Más información

[Documentación completa](../cartacolibri.md) · [README del motor Colibrì](https://github.com/JustVugg/colibri) · [Contribuir](../../CONTRIBUTING.md) · [📄 Tutorial PDF ilustrado](CartaColibri-Tutorial.es.pdf)
