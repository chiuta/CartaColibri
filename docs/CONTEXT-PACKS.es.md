# Paquetes de contexto

Con el motor propio de Colibri Companion (`corbul serve --packs DIR`), CartaColibri puede reutilizar el trabajo hecho sobre cada fuente entre llamadas: resumir, luego extraer, luego comparar sobre los mismos documentos sin releerlos.

- Se activa con `--packs` (cualquier comando que acepte `--url`) o `COLI_PACKS=1`.
- Cada fuente de un prompt se convierte en un paquete titulado por su nombre y una breve #etiqueta en lugar de su etiqueta [S#] (la etiqueta depende de la posición, así que el mismo texto cambiaría de una llamada a otra). El final del prompt enlaza cada #etiqueta con su [S#]: las citas y su comprobación funcionan como antes.
- Cualquier motor sin `/v1/packs/complete` (colibri, Ollama, servicios en la nube) se detecta una vez y se usa el chat normal. Los backends en la nube `--provider` nunca reciben paquetes.
- La respuesta es un completado de texto plano, sin la plantilla de chat: los modelos pequeños ajustados a instrucciones pueden seguirlas peor que en chat. Comprueba con `--verify`.
- Aún no medido con modelos reales.

El estante puede guardarse a mitad de tamaño y comprimido iniciando corbul con `--packs-kv f16` y `--packs-compress` (redondeo F16 en cada fila; la compresión no pierde nada).
