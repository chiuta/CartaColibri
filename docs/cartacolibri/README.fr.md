<p align="center"><img src="../../assets/cartacolibri-logo.png" width="320" alt="CartaColibri — Local AI Library"></p>

<p align="center"><b>Tes documents, interrogeables et expliqués — 100 % en local.</b></p>

<p align="center">[English](README.en.md) · [Română](README.ro.md) · <b>Français</b> · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · [Magyar](README.hu.md)</p>

CartaColibri est une bibliothèque IA locale construite sur le moteur Colibrì. Elle indexe les dossiers que tu choisis, te laisse les interroger en un éclair, puis demande à un modèle de langage local de résumer, d’extraire des données ou de synthétiser des groupes de fichiers — en citant toujours le fichier source. Rien ne quitte ton ordinateur et tes originaux ne sont jamais modifiés.

## Ce qu’elle fait

- **Bibliothèque** — tous les documents locaux avec leur emplacement ; ajoute ou retire des dossiers, envoie des fichiers, consulte-les, enregistre du texte OCR si tu veux.
- **RAG-fetch** — index plein texte incrémental façon DocFetcher (mots, « expressions », préfixe*, AND/OR/NOT, filtres par type, taille, emplacement) et **groupes** de fichiers réutilisables.
- **RAG-CARTA** — trois tâches locales sur un groupe : **résumer**, **extraire** (tableau ou JSON, « NOT FOUND » plutôt que des suppositions), **synthétiser** (thèmes, points communs, contradictions). Chaque réponse cite [S1], [S2]… et liste ses sources.
- **Privé par conception** — ni cloud ni télémétrie ; les originaux sont seulement lus. 8 langues : EN, RO, FR, IT, ES, PT, DE, HU.
- **web-grab** — un petit navigateur en Rust (clean-room, remplace Falkon) avec onglets, qui connaît ta bibliothèque : un badge quand la page est déjà enregistrée (et quand elle a changé), des documents liés pendant que tu navigues, enregistrement en un clic, questions à ton Colibrì local sur une page — ou sur tous les onglets ouverts, avec citations. Une commande et un bookmarklet couvrent les autres navigateurs. Aussi : 🖍 surlignages enregistrés avec la page, 👁 pages surveillées avec le détail des changements (`cartacolibri webwatch`), instantané HTML à chaque sauvegarde, historique consultable optionnel (`cartacolibri history on`), 🗂📝📊 résumé ou extraction sur tous les onglets, et `?question` dans la barre d'adresse pour interroger toute ta bibliothèque.

## Démarrage rapide

Compile-la (ou prends le zip portable) et lance la page locale. Pour les tâches IA, lance aussi le moteur Colibrì avec un modèle adapté à ta RAM (voir sa doc) :

```bash
git clone https://github.com/chiuta/CartaColibri
cd CartaColibri
cargo build --release                       # Rust 1.90+, or use the portable zip
./target/release/cartacolibri serve         # then open http://127.0.0.1:8765/
coli web --model ~/colibri-models/<model>   # Colibrì engine, only for the AI tasks
```

Ou utilise la ligne de commande, sans modèle pour essayer (`--mock` est un repli hors ligne) :

```bash
cartacolibri add ~/Documents
cartacolibri search "contract*" --ext pdf,docx
cartacolibri group set Contracts ~/Documents/contracts
cartacolibri synthesize "payment terms" --group Contracts --save --mock
cartacolibri grab https://example.org/article      # web-grab (command line)
cartacolibri-browser https://example.org           # web-grab (browser)
```

## Comment ça s’articule

Couche 0 — tes dossiers (lecture seule) → RAG-fetch — index, recherche, groupes → RAG-CARTA — tâches IA via le moteur Colibrì.

## Licence et crédits

CartaColibri est publié sous **licence MIT**. C’est une implémentation Rust clean-room : elle ne parle au moteur [Colibrì](https://github.com/JustVugg/colibri) (JustVugg, Apache-2.0) que via sa passerelle HTTP publique et n’en contient aucun code. La couche d’index et de recherche suit le comportement décrit dans le manuel public de DocFetcher ; aucun code DocFetcher n’est utilisé. Le navigateur web-grab s’appuie sur la webview du système et ne contient aucun code de Falkon.

## Plus d’infos

[Documentation complète](../cartacolibri.md) · [README du moteur Colibrì](https://github.com/JustVugg/colibri) · [Contribuer](../../CONTRIBUTING.md) · [📄 Tutoriel PDF illustré](CartaColibri-Tutorial.fr.pdf)
