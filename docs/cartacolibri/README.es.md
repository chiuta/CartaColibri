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
- **web-grab** — un pequeño navegador en Rust (clean-room, sustituye a Falkon) con pestañas, que conoce tu biblioteca: una insignia cuando la página ya está guardada (y cuando ha cambiado), documentos relacionados mientras navegas, guardado con un clic, preguntas a tu Colibrì local sobre una página — o sobre todas las pestañas abiertas, con citas. Un comando y un bookmarklet cubren otros navegadores. Además: 🖍 resaltados guardados con la página, 👁 páginas vigiladas con el detalle de los cambios (`cartacolibri webwatch`), instantánea HTML en cada guardado, historial con búsqueda opcional (`cartacolibri history on`), 🗂📝📊 resumen o extracción entre pestañas y `?pregunta` en la barra de direcciones para consultar toda tu biblioteca.

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

CartaColibri es software **alfa** (versión 0.2.0-alpha): funciona, pero espera algunas asperezas y guarda copias de seguridad de lo importante. Linux es la plataforma mejor probada: allí el navegador web-grab se autoprueba en CI. En **Windows y macOS** el navegador se compila en CI, pero **nunca se ha ejecutado a mano**. Las comprobaciones automáticas de GitHub (CI) **aún no están verificadas**: los trabajos se han quedado atascados en la cola. Tus documentos están a salvo en cualquier caso, porque solo se leen.

## Licencia y créditos

CartaColibri se publica bajo **licencia MIT**. Es una implementación Rust clean-room: habla con el motor [Colibrì](https://github.com/JustVugg/colibri) (JustVugg, Apache-2.0) solo a través de su pasarela HTTP pública y no contiene nada de su código. La capa de índice y búsqueda sigue el comportamiento descrito en el manual público de DocFetcher; no se usa código de DocFetcher. El navegador web-grab usa el webview del propio sistema operativo y no contiene código de Falkon.

## Más información

[Documentación completa](../cartacolibri.md) · [README del motor Colibrì](https://github.com/JustVugg/colibri) · [Contribuir](../../CONTRIBUTING.md) · [📄 Tutorial PDF ilustrado](CartaColibri-Tutorial.es.pdf)
