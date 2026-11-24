<p align="center"><img src="../../assets/cartacolibri-logo.png" width="320" alt="CartaColibri — Local AI Library"></p>

<p align="center"><b>Os teus documentos, pesquisáveis e explicados — 100 % local.</b></p>

<p align="center">[English](README.en.md) · [Română](README.ro.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · <b>Português</b> · [Deutsch](README.de.md) · [Magyar](README.hu.md)</p>

O CartaColibri é uma biblioteca de IA local construída sobre o motor Colibrì. Indexa as pastas que escolheres, deixa-te pesquisá-las num instante e depois pede a um modelo de linguagem local que resuma, extraia dados ou sintetize grupos de ficheiros — citando sempre o ficheiro de origem. Nada sai do teu computador e os teus originais nunca são modificados.

## Os teus primeiros 5 minutos

Não precisas de experiência. Três passos e já terás pesquisado e resumido os teus próprios ficheiros.

1. **Arranca-a.** Descomprime o zip portátil e executa o lançador, ou escreve `cartacolibri start` num terminal. O teu navegador mostra a página local, `http://127.0.0.1:8765/`.
2. **Adiciona uma pasta.** Na página abre **Biblioteca**, escreve o caminho de uma pasta como `~/Documents` e clica em **Adicionar pasta**. A primeira indexação de uma pasta grande pode demorar alguns minutos. Os teus ficheiros são apenas lidos, nunca alterados.
3. **Pesquisa e depois pergunta.** Abre o **RAG-fetch**, pesquisa uma palavra, assinala alguns ficheiros e cria um grupo. Depois abre o **RAG-CARTA**, escolhe **Resumir** e clica em **Executar**. Ainda não tens um modelo de IA? Assinala **Modo offline (sem modelo)** e experimenta na mesma.

Preferes o terminal? Os mesmos três passos:

```bash
cartacolibri start                 # step 1 - leave this window open (use a second one for the rest)
cartacolibri add ~/Documents       # step 2
cartacolibri group set Demo ~/Documents
cartacolibri summarize "main points" --group Demo --mock   # step 3 - --mock = no AI model needed
```

## O que faz

- **Biblioteca** — todos os documentos locais com a sua localização; adiciona ou remove pastas, envia ficheiros, vê-os e, se quiseres, guarda o texto OCR.
- **RAG-fetch** — índice de texto integral incremental ao estilo DocFetcher (palavras, «frases», prefixo*, AND/OR/NOT, filtros por tipo, tamanho e localização) e **grupos** de ficheiros reutilizáveis.
- **RAG-CARTA** — três tarefas locais sobre um grupo: **resumir**, **extrair** (tabela ou JSON, «NOT FOUND» em vez de suposições) e **sintetizar** (temas, pontos comuns, contradições). Cada resposta cita [S1], [S2]… e lista as fontes.
- **Privada por conceção** — sem nuvem nem telemetria; os originais são apenas lidos. 8 línguas: EN, RO, FR, IT, ES, PT, DE, HU.
- **web-grab** — um pequeno navegador em Rust (clean-room) com separadores, que conhece a tua biblioteca: um selo quando a página já está guardada (e quando mudou), documentos relacionados enquanto navegas, guardar com um clique, perguntas ao teu Colibrì local sobre uma página — ou sobre todos os separadores abertos, com citações. Um comando e um bookmarklet cobrem outros navegadores. Também: 🖍 destaques guardados com a página, 👁 páginas vigiadas com o detalhe das alterações (`cartacolibri webwatch`), instantâneo HTML em cada gravação, histórico pesquisável opcional (`cartacolibri history on`), 🗂📝📊 resumo ou extração entre separadores e `?pergunta` na barra de endereços para interrogar toda a biblioteca.

## Funcionalidades avançadas

- **Encontra** — uma barra de progresso durante a indexação; «Querias dizer…?» para os erros de escrita, com sugestões quando nada corresponde; os duplicados listados e um motivo claro para cada ficheiro ignorado. Escreve filtros diretamente na caixa (`ext:pdf`, `path:`, `size:>1mb`, datas `after:`/`before:`, expressões regulares `re:`), marca **Pesquisa inteligente** para acrescentar palavras afins e vê quantos há de cada tipo de ficheiro.
- **Organiza** — etiquetas, uma estrela e uma nota privada em cada documento; sobrevivem à reindexação. Vê os documentos relacionados, o que abriste recentemente e com mais frequência, e guarda pesquisas como coleções inteligentes.
- **14 tarefas** — comparar, cronologia, glossário, esquema, ações, FAQ, cartões, questionário, traduzir, explicar, contradições, além de resumir, extrair e sintetizar. Assinala **verificar citações** para marcar as frases que as fontes não sustentam e escolhe a língua da resposta.
- **Chat** — faz perguntas seguidas sobre toda a biblioteca ou sobre um grupo. As respostas citam as fontes ou dizem com honestidade «não encontrado nos teus documentos» em vez de adivinhar. Cada resposta mostra uma pontuação de **Respaldo nas fontes** e assinala as frases ou números que as fontes não sustentam. Marca «Cada pergunta à parte» para não recordar nada, ou limita os documentos a um período.
- **Partilha e protege** — exporta um grupo, uma etiqueta ou uma pesquisa em Markdown, CSV, JSON ou zip, com citações; uma vista de Estatísticas com gráficos e cronologia; uma cópia de segurança num único ficheiro com somas de verificação; Verificar/Reparar para o índice.
- **Traz coisas** — importa os marcadores do navegador, uma pasta do Obsidian, Zotero/BibTeX, CSV do Pocket/Raindrop ou exportações do Evernote, e segue feeds RSS/Atom (OPML de entrada e saída): tornam-se notas pesquisáveis. Também são indexados e-mails, calendários, legendas e cadernos.
- **Análises** — “Contradições” (mesma frase, outro número ou data), “Mapa de temas” (bolhas), “Novidades” desde uma data, “Comparar 2” documentos, “Nomes e números” (e-mails, links, valores, datas, telefones), “Dossiê” sobre um nome e “Verificar o meu texto” face à biblioteca.
- **Automatiza e liga** — `digest` repete uma pesquisa guardada e indica só as novidades (repete, junta a um ficheiro, resume ou guarda um ficheiro de e-mail); `mcp` permite que outras apps de IA pesquisem, leiam, perguntem, verifiquem e peçam dossiês na tua biblioteca, só em leitura.

## Novidades da 0.2.0-alpha.4

- **Filtros diretamente na caixa de pesquisa** — Escreve `ext:pdf`, `path:contratos`, `size:>1mb`, `after:2025-01`, `before:2025-06-30` ou `re:FAT-\d+` (expressão regular) junto às tuas palavras e os resultados estreitam-se. Funcionam na página web, na linha de comandos e através da API.
- **Pesquisa inteligente** — Marca «Pesquisa inteligente» e a tua IA local acrescenta alguns sinónimos e palavras afins (como um simples OR: as correspondências exatas continuam à frente). As palavras acrescentadas aparecem debaixo da caixa. Sem motor ligado recebes a pesquisa normal.
- **Chat de pergunta única** — Marca «Cada pergunta à parte» e o chat esquece as perguntas anteriores: cada resposta vem só dos teus documentos, com as suas fontes.
- **Um servidor MCP para outras apps de IA** — `cartacolibri mcp` permite que outras aplicações de IA pesquisem na tua biblioteca, leiam um documento e façam perguntas sobre ele através do Model Context Protocol. É só de leitura, corre no teu computador e só chega aos documentos devolvidos pela pesquisa.
- **Um resumo das novidades** — `cartacolibri digest NOME` repete uma pesquisa guardada e indica só os documentos que apareceram desde a última vez. Acrescenta `--every 60` para repetir de hora a hora, `--out relatorio.md` para juntar a um ficheiro e `--summarize` para um resumo curto da tua IA local.
- **Saber quanto confiar numa resposta** — Cada resposta do chat mostra agora uma pontuação «Respaldo nas fontes»: quanto de cada frase se encontra mesmo nas fontes citadas, com as frases a rever e os números que não aparecem em nenhuma fonte.
- **Compara dois documentos e vê todos os nomes e números** — Marca dois documentos e carrega em «Comparar 2» para ver, frase a frase, o que difere entre duas versões. «Nomes e números» lista todos os e-mails, links, valores, datas e telefones da tua biblioteca e onde aparecem. Na linha de comandos: `compare` e `entities`.
- **Deteta contradições, vê o mapa de temas, acompanha o que mudou** — «Contradições» lista frases de documentos diferentes iguais exceto num número ou data (500 contra 550). «Mapa de temas» agrupa os teus documentos por assunto. «Novidades» lista o que mudou desde uma data. Na linha de comandos: `conflicts`, `topics`, `changes --since`.
- **Verifica o teu próprio texto face à biblioteca** — Cola um rascunho em «Verificar o meu texto» e cada frase fica marcada como confirmada, em parte, números diferentes ou não encontrada, com a melhor prova dos teus documentos. «Dossiê» reúne tudo o que a tua biblioteca diz de um nome ou termo, com as suas datas e valores. Outras apps de IA acedem a ambos pelo servidor MCP.
- **Pergunta sobre um período e recebe os resumos como ficheiro de e-mail** — No chat escolhe «Documentos desde … até …» e a resposta vem só de documentos modificados nesse período (linha de comandos: `chat --since --until`). O mapa de temas é agora uma imagem de bolhas em que podes clicar. `digest --mail-to tu@example.org` guarda também cada relatório como ficheiro de e-mail na pasta outbox; acrescenta `--sendmail` para o entregar ao programa de correio do teu computador. Nada mais sai do teu computador.

## Novidades da 0.2.0-alpha.2

- **O endereço do motor funciona sozinho** — `http://127.0.0.1:8000` e `http://127.0.0.1:8000/v1` servem as duas. O CartaColibri lê a lista de modelos em `/v1/models`, escolhe sozinho o modelo carregado e já não precisa de uma página `/health`. Isto resolve a mensagem «AI engine not running (404)».
- **Encontra a tua IA sozinho** — O CartaColibri procura um motor em execução neste computador: as definições próprias do Colibri Companion, um ficheiro de descoberta e depois as portas locais habituais (8000, 8080, 11434, 1234, 8001, 5000). Só são testados endereços loopback. Abre ⚙ para ver o que foi encontrado ou carrega em «Iniciar o Colibri Companion».
- **Estado e velocidade em direto** — O cabeçalho mostra o modelo em execução e se foi encontrado automaticamente, atualizado a cada poucos segundos. Debaixo de cada resposta vês os tokens usados e a velocidade; abaixo de 3 tokens/s recebes a sugestão de escolher um modelo mais pequeno. Os pedidos seguem para o motor um de cada vez.
- **Páginas web a partir do diálogo de importação** — Em «Trazer coisas de fora» os feeds de notícias deram lugar ao web-grab: cola um endereço e guarda a página como um único ficheiro HTML ou MHTML na biblioteca. Os feeds continuam lá, recolhidos.
- **Três botões no navegador web-grab** — «Save to folder» guarda a cópia de texto na biblioteca, «Grab website HTML» guarda a página (Shift+clique para MHTML), «PDF» imprime-a em PDF. O PDF é silencioso no Linux; no Windows e no macOS abre-se a caixa de impressão e escolhes «Guardar como PDF».
- **As páginas guardadas são pesquisáveis** — Os ficheiros `.mhtml` e `.html` são indexados com o conjunto de caracteres certo, por isso uma página guardada encontra-se como qualquer documento. As partes de uma página em endereços privados nunca são descarregadas nem ligadas.

## Início rápido

Compila-a (ou obtém o zip portátil) e arranca a página local. Para as tarefas de IA, arranca também o motor Colibrì com um modelo que caiba na tua RAM (vê a documentação dele):

```bash
git clone https://github.com/chiuta/CartaColibri
cd CartaColibri
cargo build --release                       # Rust 1.90+, or use the portable zip
./target/release/cartacolibri serve         # then open http://127.0.0.1:8765/
coli web --model ~/colibri-models/<model>   # Colibrì engine, only for the AI tasks
```

Ou usa a linha de comandos, sem modelo para experimentar (`--mock` é um modo offline):

```bash
cartacolibri add ~/Documents
cartacolibri search "contract*" --ext pdf,docx
cartacolibri group set Contracts ~/Documents/contracts
cartacolibri synthesize "payment terms" --group Contracts --save --mock
cartacolibri grab https://example.org/article      # web-grab (command line)
cartacolibri-browser https://example.org           # web-grab (browser)
```

## Mais comandos

| Comando | O que faz |
|---|---|
| `start [--folder F] [--port N] [--no-open]` | Arranque guiado: escolhes uma pasta, é indexada e abre-se a página web. |
| `doctor` | Verifica o motor, o OCR, as ferramentas PDF, o disco e a porta e indica a correção exata de cada problema. |
| `add FOLDER`, `remove FOLDER`, `folders`, `index [FOLDER]`, `problems` | Regista ou esquece uma pasta (os ficheiros nunca são tocados), lista as pastas, reindexa e lista os ficheiros que não foi possível ler. |
| `watch [--interval S]` | Mantém o índice em dia: volta a analisar as pastas a cada S segundos. |
| `search QUERY [--ext pdf,txt] [--min-kb N] [--max-kb N] [--path P] [--group G] [--json]` | Pesquisa na biblioteca. Palavras, «frases», prefixo*, AND/OR/NOT e filtros na própria consulta: `ext:pdf` `path:contratos` `size:>1mb` `after:2025-01` `before:2025-06-30` `re:FAT-\d+`. |
| `list [--group G] [--json]`, `group set/list/show/rm/from-search` | Lista os documentos; os grupos são conjuntos de documentos com nome onde pesquisar ou sobre os quais perguntar. |
| `summarize`, `extract`, `synthesize` (same options as `task`) | Atalhos para as três tarefas principais: resumir cada documento, extrair campos ou combinar vários numa resposta. |
| `grab URL`, `bookmarklet` | Guarda uma página web na biblioteca (HTML, MHTML ou PDF); o bookmarklet faz isso em qualquer navegador. |
| `serve [--port N] [--url ENGINE]` | Inicia a página web neste computador (http://127.0.0.1:PORT). |
| `completions bash\|zsh\|fish\|powershell`, `import-json FILE` | Imprime um script de autocompletar com Tab para a tua shell; reimporta etiquetas, estrelas, notas e grupos exportados em JSON. |
| `task NAME [request] --group G \| --tag T \| --query Q [--mock] [--verify] [--lang xx] [--save md\|html\|txt]` | as 14 tarefas (compare, timeline, glossary, outline, actions, faq, flashcards, quiz, translate, explain, contradictions, summarize, extract, synthesize); `--verify` marca as frases cujo `[S#]` o texto não sustenta |
| `chat [--group G] [--since DATE] [--until DATE] [--lang xx]` | perguntas seguidas sobre a biblioteca; respostas citadas, ou «não encontrado nos teus documentos» |
| `tag add/rm/list/show`, `star`, `note` | a tua organização pessoal, mantida após reindexar |
| `saved add/list/run/rm` | pesquisas guardadas / coleções inteligentes |
| `recent`, `related PATH` | o que abriste recentemente; documentos semelhantes |
| `export --format md\|csv\|json\|zip` | partilha um grupo, uma etiqueta ou uma pesquisa guardada, com citações |
| `backup OUT [--index]`, `backup-check ZIP`, `restore ZIP` | cópia de segurança num único ficheiro com somas de verificação; verifica-a; restaura-a |
| `verify [--repair]` | verifica o índice; com `--repair` reconstrói-o a partir dos teus ficheiros |
| `stats`, `timeline` | números da tua biblioteca; documentos por mês |
| `import PATH` | marcadores `.html`, pasta do Obsidian, `.bib`, Pocket/Raindrop `.csv`, Evernote `.enex` |
| `feeds add/list/rm/poll`, `feeds opml-import/opml-export` | subscrições RSS/Atom guardadas como notas |
| `webwatch`, `history` | páginas web vigiadas com diferenças; histórico de navegação opcional |
| `digest NAME [--every MIN] [--out FILE.md] [--summarize] [--all] [--mail-to ADDR [--sendmail]]` | Repete uma pesquisa guardada e indica só os documentos novos desde a última vez; pode repetir-se, juntar a um ficheiro, resumir com a tua IA local ou guardar um ficheiro de e-mail na pasta outbox. |
| `entities [--kind email\|url\|money\|date\|phone] [--group G] [--json]` | Todos os e-mails, links, valores, datas e telefones dos teus documentos, com os documentos onde aparecem. |
| `compare A B [--json]` | Compara dois documentos frase a frase: quão parecidos são e o que existe só num deles (A e B são caminhos ou o seu final). |
| `conflicts [--group G] [--json]` | Pares de frases de documentos diferentes iguais exceto num número ou data (500 contra 550): candidatos a verificar, não veredictos. |
| `topics [--json]` | Agrupa os documentos por assunto (as suas palavras mais características). |
| `dossier TERM [--json]` | Tudo o que a biblioteca diz de um nome ou termo: passagens por documento, mais as datas e valores encontrados. |
| `changes --since DATE [--json]` | Documentos modificados desde uma data (YYYY, YYYY-MM ou YYYY-MM-DD), os mais recentes primeiro. |
| `factcheck FILE\|- [--json]` | Verifica um texto teu face à biblioteca: cada frase fica marcada como confirmada, em parte, números diferentes ou não encontrada, com a melhor prova. |
| `mcp [--url ENGINE] [--mock]` | Servidor MCP por stdio para que outras apps de IA pesquisem, leiam, perguntem, verifiquem e peçam dossiês na tua biblioteca (só leitura). |

## Como tudo encaixa

Camada 0 — as tuas pastas (só leitura) → RAG-fetch — índice, pesquisa, grupos → RAG-CARTA — tarefas de IA através do motor Colibrì.

## Perguntas frequentes

**O que significa «local»?**  
Tudo corre no teu próprio computador: a página, o índice de pesquisa e o modelo de IA. Funciona mesmo sem rede. A única exceção é o web-grab, que descarrega as páginas web que lhe pedes.

**Os meus dados são enviados para algum lado?**  
Não. Não há nuvem, conta, telemetria nem estatísticas de utilização. O único tráfego que sai da tua máquina são as páginas web que pedes explicitamente ao web-grab para obter. Detalhes em «Privacidade e segurança».

**O Colibrì não está a correr. Posso usar na mesma?**  
Sim. A Biblioteca e a pesquisa nunca precisam de modelo. Para resumir, extrair e sintetizar, assinala **Modo offline (sem modelo)** (`--mock` na linha de comandos): obténs uma resposta simples feita a partir dos teus ficheiros, com citações, mas sem redação real de IA. Para respostas a sério, arranca o motor Colibrì.

**O que é o Colibrì e preciso dele?**  
O [Colibrì](https://github.com/JustVugg/colibri) é um motor local separado que executa o modelo de linguagem. Só precisas dele para as respostas de IA. Escolhe um modelo cujo ficheiro seja mais pequeno do que a tua RAM livre e começa por um pequeno. Vê a documentação do Colibrì.

**Que ficheiros consegue ler?**  
Texto e notas (txt, md, rst, log, csv, json, yaml, html, rtf), PDF, ficheiros Word, OpenDocument, PowerPoint e Excel (docx, odt, pptx, xlsx), epub e, com o OCR ligado, imagens e PDF digitalizados.

**E as digitalizações e fotografias de documentos?**  
Liga o OCR (`--ocr` ou `CARTACOLIBRI_OCR=1`). Precisa de `tesseract` instalado no teu computador, mais `pdftoppm` para PDF digitalizados. A página avisa-te quando falta o tesseract.

**Como atualizo?**  
Versão portátil: descarrega o zip mais recente e descomprime-o. A partir do código-fonte: `git pull` e depois `cargo build --release`. Os teus dados em `~/.cartacolibri` ficam onde estão e o índice atualiza-se sozinho, só no que mudou.

**Como removo tudo?**  
Apaga a pasta do programa e depois a pasta de estado `~/.cartacolibri` (ou a que `CARTACOLIBRI_HOME` indicar). É tudo: os teus documentos nunca foram tocados.

**Algo de especial no Windows ou macOS?**  
O zip portátil existe para ambos, mas o Linux é o sistema mais bem testado. O navegador web-grab é compilado em CI para Windows e macOS e lá nunca foi executado à mão. No Windows precisa do runtime WebView2. Se o sistema te pedir para confirmares a abertura de um programa descarregado, confirma só se o obtiveste na página do projeto.

**Porquê «alfa»?**  
Porque é jovem: o núcleo funciona e está testado, mas não em todos os sistemas, e as verificações automáticas no GitHub ainda não estão confirmadas. Conta com algumas arestas e diz-nos quais.

**Como sobrevivem as etiquetas, estrelas e notas à reindexação?**  
Não ficam guardadas no índice de pesquisa, mas num pequeno ficheiro à parte, `organize.json`, na tua pasta `<home>`, ligado ao caminho de cada documento. Por isso **Atualizar índice**, uma reindexação completa e até **Verificar/Reparar** (que reconstrói o índice) não lhes tocam. Se mudares o nome ou moveres um ficheiro, conta como documento novo e as etiquetas ficam no caminho antigo.

**Como faço uma cópia de segurança e a restauro?**  
Abre **Estatísticas** e carrega em **Transferir cópia de segurança** (assinala a caixa para incluir o índice), ou executa `cartacolibri backup my.zip`. É um único zip com somas de verificação, com as tuas etiquetas, notas, grupos, pesquisas guardadas e notas importadas, nunca os teus documentos originais. `cartacolibri backup-check my.zip` testa-o e `cartacolibri restore my.zip` repõe tudo. Guarda o zip noutro sítio que não este computador.

**Como importo marcadores, Obsidian ou Zotero?**  
Clica em **Trazer coisas** na Biblioteca, ou executa `cartacolibri import CAMINHO`. Entende uma exportação de marcadores do navegador (`.html`), uma pasta do Obsidian, um ficheiro Zotero/BibTeX `.bib`, ficheiros `.csv` do Pocket ou Raindrop e exportações do Evernote `.enex`. Cada item torna-se uma pequena nota na tua biblioteca, que podes pesquisar, etiquetar e citar. Os teus originais são apenas lidos.

**Como funcionam os feeds?**  
Em **Trazer coisas → Feeds**, cola um endereço RSS ou Atom e carrega em **Adicionar**. Quando carregas em **Verificar agora** (ou executas `cartacolibri feeds poll`), as novidades são guardadas como notas e indexadas, por isso podes pesquisá-las e fazer perguntas sobre elas. Nada corre sozinho em segundo plano. Podes trazer e levar as tuas subscrições em OPML.

**O que faz «verificar citações»?**  
Depois de a IA responder, o CartaColibri relê cada frase. Se uma frase não tem citação `[S#]`, ou a fonte citada não contém realmente o que a frase diz, fica marcada `[unsupported]`. É uma rede de segurança rápida, não uma garantia: abre a fonte para tudo o que for importante.

**Porque foi recusada uma página web?**  
O web-grab só obtém páginas públicas. Os endereços da tua própria máquina ou de uma rede privada (`localhost`, `192.168.x.x`, `10.x.x.x` e semelhantes) são recusados, para que nenhuma página web o possa enganar e levá-lo a ler os teus serviços internos. Se mesmo assim quiseres, por exemplo para a tua intranet, arranca o CartaColibri com `CARTACOLIBRI_GRAB_LOCAL=1`.

## Quando algo corre mal

Experimenta primeiro `cartacolibri doctor`: verifica a tua instalação e diz-te o que corrigir.

| Sintoma | Causa provável | O que fazer |
|---|---|---|
| `cartacolibri: command not found` | A pasta do programa não está no teu PATH. | Executa-o com o caminho completo, por exemplo `./target/release/cartacolibri`, ou usa o lançador do zip portátil. |
| A página não abre em `http://127.0.0.1:8765/` | A página local não está a correr, ou a porta está ocupada. | Executa `cartacolibri start`. Se a porta estiver ocupada, executa `cartacolibri serve --port 8766` e abre esse endereço. |
| Não sabes o que está mal | Falta algo na configuração. | Executa `cartacolibri doctor`: verifica a configuração e diz-te o que corrigir. |
| As tarefas de IA dizem que não chegam ao Colibrì | O motor Colibrì não está a correr, ou escuta noutro endereço (por omissão `http://127.0.0.1:8000`). | Arranca-o (`coli web --model ~/colibri-models/<model>`) ou indica o endereço com `--url`. Entretanto, assinala **Modo offline (sem modelo)**. |
| Os ficheiros novos não aparecem na pesquisa | O índice ainda não os viu. | Clica em **Atualizar índice** ou executa `cartacolibri index`. O `cartacolibri watch` mantém-no atualizado automaticamente. |
| Alguns ficheiros aparecem em «Ficheiros ignorados» | Estão corrompidos, protegidos, não suportados ou são digitalizações. | Executa `cartacolibri problems` para ver o motivo. Para digitalizações, liga o OCR. |
| Digitalizações e imagens não dão texto | O OCR está desligado ou o `tesseract` não está instalado. | Instala o `tesseract` (e o `pdftoppm` para PDF) e acrescenta `--ocr`, ou define `CARTACOLIBRI_OCR=1`. |
| A resposta diz NOT FOUND | Os ficheiros escolhidos não contêm a informação. É de propósito: nunca adivinha. | Confirma que está escolhido o grupo certo, reformula a pergunta ou acrescenta ao grupo os ficheiros que faltam. |
| O navegador web-grab não abre | Falta um componente do sistema. | Linux: instala `libwebkit2gtk-4.1`. Windows: instala o runtime WebView2. Entretanto, `cartacolibri grab URL` funciona sem ele. |
| O teu próprio script recebe `403 missing X-Colibri-UI header` | É uma verificação de segurança, não um erro. | Envia o cabeçalho `X-Colibri-UI: 1` em cada chamada a `/carta/*`. |
| O índice parece estragado: resultados estranhos, ficheiros em falta, um erro sobre a base de dados | O ficheiro do índice ficou danificado (por exemplo por uma falha ou um disco cheio) ou perdeu a sincronia com as tuas pastas. | Abre **Estatísticas → Verificar / Reparar**, ou executa `cartacolibri verify --repair`. Reconstrói o índice a partir dos teus ficheiros; as etiquetas, notas e grupos mantêm-se. |
| Um ficheiro foi ignorado por ser «demasiado lento» | A leitura excedeu o limite de tempo (120 segundos por omissão), normalmente um PDF enorme ou danificado, ou uma digitalização muito grande. | Aumenta o limite, por exemplo `CARTACOLIBRI_EXTRACT_TIMEOUT=600`, e depois **Atualizar índice**. Ou divide / volta a exportar o ficheiro. O motivo aparece em «Ficheiros ignorados». |
| «AI engine not running … (404)» | O endereço aponta para algo que não é o motor, ou não há nenhum modelo carregado. | Abre ⚙, carrega em «Procurar o motor» e escolhe o endereço listado; servem `…:8000` e `…:8000/v1`. Confirma que há um modelo carregado no Colibri Companion. |

## Glossário em palavras simples

- **Índice** — Um catálogo das palavras dos teus ficheiros, guardado para que uma pesquisa demore um piscar de olhos em vez de reler tudo.
- **RAG** — «Retrieval-augmented generation»: primeiro encontram-se as passagens relevantes nos teus ficheiros e depois a IA responde usando só essas.
- **Citação [S1]** — Uma marca como [S1] ou [S2] numa resposta. Aponta para o ficheiro de origem da frase, para poderes confirmar.
- **Grupo** — Uma lista com nome de ficheiros (um projeto, um tema) em que podes pesquisar ou perguntar vezes sem conta.
- **Gateway (porta de ligação)** — O pequeno endereço web (por omissão `http://127.0.0.1:8000`) pelo qual o CartaColibri fala com o motor Colibrì.
- **Instantâneo (snapshot)** — Uma cópia HTML guardada de uma página web, feita sempre que a guardas com o web-grab, para manteres o que ela dizia nesse dia.
- **Watch (vigilância)** — Duas coisas: `cartacolibri watch` mantém o índice atualizado, e `cartacolibri webwatch` volta a obter as páginas que segues e mostra o que mudou.

## Privacidade e segurança

- **Onde ficam as coisas.** Tudo o que o CartaColibri escreve vive numa só pasta, `<home>`: por omissão `~/.cartacolibri`, ou a pasta indicada em `CARTACOLIBRI_HOME`. Lá dentro: `index.sqlite` (o índice de pesquisa), `groups.json`, `library/uploads/` (os ficheiros que envias), `outputs/` (os resultados que escolheste guardar), `snapshots/` e `web.json` (as páginas web guardadas), `highlights.json`, `watch.json`, `settings.json` e `clip.token` (um segredo para o bookmarklet: não o partilhes).
- **Os teus originais são só de leitura.** O CartaColibri lê os teus documentos e nunca os altera, move ou apaga.
- **A página é só local.** O servidor escuta apenas em `127.0.0.1`, por isso os outros computadores da tua rede não lhe chegam. Também recusa pedidos cujo endereço ou origem não seja local.
- **O cabeçalho `X-Colibri-UI`.** Cada chamada da página ao seu próprio servidor leva este cabeçalho. Uma página web visitada noutro separador não o consegue acrescentar, por isso não pode fazer o teu navegador tocar na aplicação local nas tuas costas.
- **Nada sai de `127.0.0.1`, exceto as páginas que pedes para obter.** Sem telemetria, sem estatísticas, sem nuvem. O web-grab só obtém os endereços que lhe dás (e as páginas que escolheste vigiar). O site de apresentação também não carrega nada de outros servidores.
- **O histórico vem desligado.** O histórico do navegador web-grab só existe se o ligares (`cartacolibri history on`), fica no teu disco, e `cartacolibri history clear` apaga-o.

## Estado real

O CartaColibri é software **alfa** (versão 0.2.0-alpha): funciona, mas conta com algumas arestas por limar e guarda cópias de segurança do que é importante. O Linux é a plataforma mais testada: lá o navegador web-grab é autotestado em CI. No **Windows e no macOS** o navegador é compilado em CI, mas **nunca foi executado à mão**. As verificações automáticas no GitHub (CI) passam no Linux, no Windows e no macOS. Os teus documentos estão seguros de qualquer forma, porque são apenas lidos. As transferências portáteis para Linux, Windows e macOS estão na página **Releases** do GitHub (marcadas como pré-versão).

## Segurança

Encontraste uma vulnerabilidade? Comunica-a em privado, como descrito em [SECURITY.md](../../SECURITY.md); por favor, não abras primeiro uma issue pública. A forma como o servidor se defende (só loopback, cabeçalho `X-Colibri-UI`, sem endereços privados no web-grab, originais só de leitura) está descrita em [docs/threat-model.md](../threat-model.md).

## Licença e créditos

O CartaColibri é publicado sob **licença MIT**. É uma implementação Rust clean-room: fala com o motor [Colibrì](https://github.com/JustVugg/colibri) (JustVugg, Apache-2.0) apenas através do seu gateway HTTP público e não contém código dele. A camada de índice e pesquisa segue o comportamento descrito no manual público do DocFetcher; não é usado código do DocFetcher. O navegador web-grab usa o webview do próprio sistema operativo e não contém código do Falkon.

**Alexandru-Ionuț Chiuță** · **Alexandru Dumbrava** (coautor)

## Mais informação

[Documentação completa](../cartacolibri.md) · [README do motor Colibrì](https://github.com/JustVugg/colibri) · [Contribuir](../../CONTRIBUTING.md) · [📄 Tutorial PDF ilustrado](CartaColibri-Tutorial.pt.pdf)
