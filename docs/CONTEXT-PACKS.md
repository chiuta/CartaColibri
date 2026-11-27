# Context packs

With Colibri Companion's own engine (`corbul serve --packs DIR`), CartaColibri can reuse the work done on each source across calls: summarize, then extract, then compare over the same documents without reading them again.

- Turn it on with `--packs` (any command that takes `--url`) or `COLI_PACKS=1`.
- Each source of a prompt becomes a pack headed by its name and a short #tag instead of its [S#] label (the label depends on the position, so the same text would differ from call to call). The tail of the prompt maps each #tag to its [S#] label, so the citations and their checks work as before.
- Any engine without `/v1/packs/complete` (colibri, Ollama, cloud services) is detected once and the ordinary chat is used. Cloud `--provider` backends never receive packs.
- The answer is a plain-text completion, without the model's chat template: small instruction-tuned models may follow the instructions less well than in chat. Check with `--verify`.
- Not measured on real models yet.

The shelf can be kept at half the size and compressed by starting corbul with `--packs-kv f16` and `--packs-compress` (F16 rounding on every row; the compression is lossless).
