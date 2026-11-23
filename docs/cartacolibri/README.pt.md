<p align="center"><img src="../../assets/cartacolibri-logo.png" width="320" alt="CartaColibri — Local AI Library"></p>

<p align="center"><b>Os teus documentos, pesquisáveis e explicados — 100 % local.</b></p>

<p align="center">[English](README.en.md) · [Română](README.ro.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · <b>Português</b> · [Deutsch](README.de.md) · [Magyar](README.hu.md)</p>

O CartaColibri é uma biblioteca de IA local construída sobre o motor Colibrì. Indexa as pastas que escolheres, deixa-te pesquisá-las num instante e depois pede a um modelo de linguagem local que resuma, extraia dados ou sintetize grupos de ficheiros — citando sempre o ficheiro de origem. Nada sai do teu computador e os teus originais nunca são modificados.

## O que faz

- **Biblioteca** — todos os documentos locais com a sua localização; adiciona ou remove pastas, envia ficheiros, vê-os e, se quiseres, guarda o texto OCR.
- **RAG-fetch** — índice de texto integral incremental ao estilo DocFetcher (palavras, «frases», prefixo*, AND/OR/NOT, filtros por tipo, tamanho e localização) e **grupos** de ficheiros reutilizáveis.
- **RAG-CARTA** — três tarefas locais sobre um grupo: **resumir**, **extrair** (tabela ou JSON, «NOT FOUND» em vez de suposições) e **sintetizar** (temas, pontos comuns, contradições). Cada resposta cita [S1], [S2]… e lista as fontes.
- **Privada por conceção** — sem nuvem nem telemetria; os originais são apenas lidos. 8 línguas: EN, RO, FR, IT, ES, PT, DE, HU.

## Início rápido

Obtém o código e arranca o motor com um modelo que caiba na tua RAM (vê a documentação do Colibrì); depois abre a página do CartaColibri:

```bash
git clone https://github.com/chiuta/CartaColibri
cd CartaColibri
./start-here.sh            # Windows: START-HERE.bat
coli web --model ~/colibri-models/<model>   # then open http://127.0.0.1:8790/carta.html
```

Ou usa a linha de comandos, sem modelo para experimentar (`--mock` é um modo offline):

```bash
coli carta add ~/Documents
coli carta search "contract*" --ext pdf,docx
coli carta group set Contracts ~/Documents/contracts
coli carta synthesize "payment terms" --group Contracts --save --mock
```

## Como tudo encaixa

Camada 0 — as tuas pastas (só leitura) → RAG-fetch — índice, pesquisa, grupos → RAG-CARTA — tarefas de IA através do motor Colibrì.

## Licença e créditos

O CartaColibri é um fork do [Colibrì](https://github.com/JustVugg/colibri) de JustVugg e mantém a licença Apache-2.0. A camada de índice e pesquisa é um desenho clean-room inspirado no comportamento descrito no manual público do DocFetcher; não é usado código do DocFetcher.

## Mais informação

[Documentação completa](../cartacolibri.md) · [README do motor Colibrì](../../README.colibri.md) · [Contribuir](../../CONTRIBUTING-CARTA.md) · [📄 Tutorial PDF ilustrado](CartaColibri-Tutorial.pt.pdf)
