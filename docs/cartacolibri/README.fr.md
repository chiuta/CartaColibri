<p align="center"><img src="../../assets/cartacolibri-logo.png" width="320" alt="CartaColibri — Local AI Library"></p>

<p align="center"><b>Tes documents, interrogeables et expliqués — 100 % en local.</b></p>

<p align="center">[English](README.en.md) · [Română](README.ro.md) · <b>Français</b> · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · [Magyar](README.hu.md)</p>

CartaColibri est une bibliothèque IA locale construite sur le moteur Colibrì. Elle indexe les dossiers que tu choisis, te laisse les interroger en un éclair, puis demande à un modèle de langage local de résumer, d’extraire des données ou de synthétiser des groupes de fichiers — en citant toujours le fichier source. Rien ne quitte ton ordinateur et tes originaux ne sont jamais modifiés.

## Ce qu’elle fait

- **Bibliothèque** — tous les documents locaux avec leur emplacement ; ajoute ou retire des dossiers, envoie des fichiers, consulte-les, enregistre du texte OCR si tu veux.
- **RAG-fetch** — index plein texte incrémental façon DocFetcher (mots, « expressions », préfixe*, AND/OR/NOT, filtres par type, taille, emplacement) et **groupes** de fichiers réutilisables.
- **RAG-CARTA** — trois tâches locales sur un groupe : **résumer**, **extraire** (tableau ou JSON, « NOT FOUND » plutôt que des suppositions), **synthétiser** (thèmes, points communs, contradictions). Chaque réponse cite [S1], [S2]… et liste ses sources.
- **Privé par conception** — ni cloud ni télémétrie ; les originaux sont seulement lus. 8 langues : EN, RO, FR, IT, ES, PT, DE, HU.

## Démarrage rapide

Récupère le code et lance le moteur avec un modèle adapté à ta RAM (voir la doc Colibrì), puis ouvre la page CartaColibri :

```bash
git clone https://github.com/chiuta/CartaColibri
cd CartaColibri
./start-here.sh            # Windows: START-HERE.bat
coli web --model ~/colibri-models/<model>   # then open http://127.0.0.1:8790/carta.html
```

Ou utilise la ligne de commande, sans modèle pour essayer (`--mock` est un repli hors ligne) :

```bash
coli carta add ~/Documents
coli carta search "contract*" --ext pdf,docx
coli carta group set Contracts ~/Documents/contracts
coli carta synthesize "payment terms" --group Contracts --save --mock
```

## Comment ça s’articule

Couche 0 — tes dossiers (lecture seule) → RAG-fetch — index, recherche, groupes → RAG-CARTA — tâches IA via le moteur Colibrì.

## Licence et crédits

CartaColibri est un fork de [Colibrì](https://github.com/JustVugg/colibri) par JustVugg et conserve sa licence Apache-2.0. La couche d’index et de recherche est une conception « clean-room » inspirée du comportement décrit dans le manuel public de DocFetcher ; aucun code de DocFetcher n’est utilisé.

## Plus d’infos

[Documentation complète](../cartacolibri.md) · [README du moteur Colibrì](../../README.colibri.md) · [Contribuer](../../CONTRIBUTING-CARTA.md)
