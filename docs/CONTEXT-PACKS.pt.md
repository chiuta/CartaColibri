# Pacotes de contexto

Com o motor próprio do Colibri Companion (`corbul serve --packs DIR`), o CartaColibri pode reaproveitar o trabalho feito sobre cada fonte entre chamadas: resumir, depois extrair, depois comparar sobre os mesmos documentos sem os reler.

- Ativa-se com `--packs` (qualquer comando que aceite `--url`) ou `COLI_PACKS=1`.
- Cada fonte de um prompt torna-se um pacote intitulado pelo nome e por uma curta #etiqueta em vez da etiqueta [S#] (a etiqueta depende da posição, por isso o mesmo texto mudaria de chamada para chamada). O fim do prompt liga cada #etiqueta à sua [S#]: as citações e a sua verificação funcionam como antes.
- Qualquer motor sem `/v1/packs/complete` (colibri, Ollama, serviços na nuvem) é detetado uma vez e usa-se o chat normal. Os backends na nuvem `--provider` nunca recebem pacotes.
- A resposta é uma completação de texto simples, sem o modelo de chat: modelos pequenos ajustados a instruções podem segui-las pior do que no chat. Verifique com `--verify`.
- Ainda não medido em modelos reais.

A prateleira pode ficar a metade do tamanho e comprimida iniciando o corbul com `--packs-kv f16` e `--packs-compress` (arredondamento F16 em cada linha; a compressão não perde nada).

Se as fontes não couberem na janela do modelo, `--packs-shrink` (ou `COLI_PACKS_SHRINK=1`) deixa o corbul reduzi-las às linhas mais importantes em vez de falhar. Perde-se informação; veja `docs/SHRINK-MAP.pt.md` do Colibri Companion.
