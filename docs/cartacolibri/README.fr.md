<p align="center"><img src="../../assets/cartacolibri-logo.png" width="320" alt="CartaColibri — Local AI Library"></p>

<p align="center"><b>Tes documents, interrogeables et expliqués — 100 % en local.</b></p>

<p align="center">[English](README.en.md) · [Română](README.ro.md) · <b>Français</b> · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · [Magyar](README.hu.md)</p>

CartaColibri est une bibliothèque IA locale construite sur le moteur Colibrì. Elle indexe les dossiers que tu choisis, te laisse les interroger en un éclair, puis demande à un modèle de langage local de résumer, d’extraire des données ou de synthétiser des groupes de fichiers — en citant toujours le fichier source. Rien ne quitte ton ordinateur et tes originaux ne sont jamais modifiés.

## Tes 5 premières minutes

Aucune expérience requise. Trois étapes et tu auras interrogé et résumé tes propres fichiers.

1. **Lance-la.** Décompresse le zip portable et lance le lanceur, ou tape `cartacolibri start` dans un terminal. Ton navigateur affiche la page locale, `http://127.0.0.1:8765/`.
2. **Ajoute un dossier.** Sur la page, ouvre **Bibliothèque**, saisis le chemin d’un dossier comme `~/Documents` et clique sur **Ajouter un dossier**. La première indexation d’un gros dossier peut prendre quelques minutes. Tes fichiers sont seulement lus, jamais modifiés.
3. **Cherche, puis demande.** Ouvre **RAG-fetch**, cherche un mot, coche quelques fichiers et crée un groupe. Ouvre ensuite **RAG-CARTA**, choisis **Résumer** et clique sur **Lancer**. Pas encore de modèle IA ? Coche **Mode hors ligne (sans modèle)** et essaie quand même.

Tu préfères le terminal ? Les mêmes trois étapes :

```bash
cartacolibri start                 # step 1 - leave this window open (use a second one for the rest)
cartacolibri add ~/Documents       # step 2
cartacolibri group set Demo ~/Documents
cartacolibri summarize "main points" --group Demo --mock   # step 3 - --mock = no AI model needed
```

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

## Questions fréquentes

**Que veut dire « local » ?**  
Tout tourne sur ton propre ordinateur : la page, l’index de recherche et le modèle d’IA. Ça marche même sans réseau. Seule exception : web-grab, qui télécharge les pages web que tu lui demandes.

**Mes données sont-elles envoyées quelque part ?**  
Non. Pas de cloud, pas de compte, pas de télémétrie, pas de statistiques d’usage. Le seul trafic qui quitte ta machine, ce sont les pages web que tu demandes explicitement à web-grab de récupérer. Détails dans « Confidentialité et sécurité ».

**Colibrì ne tourne pas. Puis-je quand même m’en servir ?**  
Oui. La Bibliothèque et la recherche n’ont jamais besoin de modèle. Pour résumer, extraire et synthétiser, coche **Mode hors ligne (sans modèle)** (`--mock` en ligne de commande) : tu obtiens une réponse simple, construite à partir de tes fichiers, avec citations, mais sans vraie formulation IA. Pour de vraies réponses, lance le moteur Colibrì.

**Qu’est-ce que Colibrì, et en ai-je besoin ?**  
[Colibrì](https://github.com/JustVugg/colibri) est un moteur local distinct qui fait tourner le modèle de langage. Tu n’en as besoin que pour les réponses IA. Choisis un modèle dont le fichier est plus petit que ta RAM libre, et commence par un petit. Vois la doc de Colibrì.

**Quels fichiers peut-elle lire ?**  
Texte et notes (txt, md, rst, log, csv, json, yaml, html, rtf), PDF, fichiers Word, OpenDocument, PowerPoint et Excel (docx, odt, pptx, xlsx), epub et, avec l’OCR activé, images et PDF numérisés.

**Et les scans et photos de documents ?**  
Active l’OCR (`--ocr`, ou `CARTACOLIBRI_OCR=1`). Il faut `tesseract` installé sur ton ordinateur, plus `pdftoppm` pour les PDF numérisés. La page te prévient quand tesseract manque.

**Comment mettre à jour ?**  
Version portable : télécharge le zip plus récent et décompresse-le. Depuis les sources : `git pull`, puis `cargo build --release`. Tes données dans `~/.cartacolibri` restent en place et l’index se rafraîchit tout seul, uniquement pour ce qui a changé.

**Comment tout supprimer ?**  
Supprime le dossier du programme, puis le dossier d’état `~/.cartacolibri` (ou celui vers lequel pointe `CARTACOLIBRI_HOME`). C’est tout : tes propres documents n’ont jamais été touchés.

**Quelque chose de particulier sous Windows ou macOS ?**  
Le zip portable existe pour les deux, mais Linux est le système le mieux testé. Le navigateur web-grab est compilé en CI pour Windows et macOS et n’y a jamais été lancé à la main. Sous Windows, il a besoin du runtime WebView2. Si ton système te demande de confirmer l’ouverture d’un programme téléchargé, ne confirme que si tu l’as pris sur la page du projet.

**Pourquoi « alpha » ?**  
Parce qu’elle est jeune : le cœur fonctionne et est testé, mais pas sur tous les systèmes, et les vérifications automatiques sur GitHub ne sont pas encore confirmées. Attends-toi à des aspérités, et signale-les-nous.

## Quand quelque chose ne va pas

Essaie d’abord `cartacolibri doctor` : il vérifie ton installation et t’indique quoi corriger.

| Symptôme | Cause probable | Que faire |
|---|---|---|
| `cartacolibri: command not found` | Le dossier du programme n’est pas dans ton PATH. | Lance-le avec son chemin complet, par exemple `./target/release/cartacolibri`, ou utilise le lanceur du zip portable. |
| La page ne s’ouvre pas sur `http://127.0.0.1:8765/` | La page locale n’est pas lancée, ou le port est pris. | Lance `cartacolibri start`. Si le port est occupé, lance `cartacolibri serve --port 8766` et ouvre cette adresse. |
| Tu ne sais pas ce qui cloche | Un élément de la configuration manque. | Lance `cartacolibri doctor` : il vérifie la configuration et te dit quoi corriger. |
| Les tâches IA disent ne pas joindre Colibrì | Le moteur Colibrì ne tourne pas, ou écoute sur une autre adresse (par défaut `http://127.0.0.1:8000`). | Lance-le (`coli web --model ~/colibri-models/<model>`) ou donne son adresse avec `--url`. En attendant, coche **Mode hors ligne (sans modèle)**. |
| Les nouveaux fichiers n’apparaissent pas dans la recherche | L’index ne les a pas encore vus. | Clique sur **Mettre à jour l’index** ou lance `cartacolibri index`. `cartacolibri watch` le garde à jour automatiquement. |
| Des fichiers sont listés dans « Fichiers ignorés » | Ils sont corrompus, protégés, non pris en charge ou numérisés. | Lance `cartacolibri problems` pour voir pourquoi. Pour les scans, active l’OCR. |
| Les scans et images ne donnent aucun texte | L’OCR est désactivé ou `tesseract` n’est pas installé. | Installe `tesseract` (et `pdftoppm` pour les PDF) et ajoute `--ocr`, ou définis `CARTACOLIBRI_OCR=1`. |
| La réponse dit NOT FOUND | Les fichiers choisis ne contiennent pas l’information. C’est voulu : elle ne devine jamais. | Vérifie que le bon groupe est sélectionné, reformule la question ou ajoute au groupe les fichiers manquants. |
| Le navigateur web-grab ne s’ouvre pas | Un composant système manque. | Linux : installe `libwebkit2gtk-4.1`. Windows : installe le runtime WebView2. En attendant, `cartacolibri grab URL` fonctionne sans lui. |
| Ton propre script reçoit `403 missing X-Colibri-UI header` | C’est un contrôle de sécurité, pas un bug. | Envoie l’en-tête `X-Colibri-UI: 1` avec chaque appel à `/carta/*`. |

## Glossaire en mots simples

- **Index** — Un catalogue des mots présents dans tes fichiers, gardé pour qu’une recherche prenne un clin d’œil au lieu de tout relire.
- **RAG** — « Retrieval-augmented generation » : on trouve d’abord les passages pertinents dans tes fichiers, puis l’IA répond en s’appuyant uniquement sur eux.
- **Citation [S1]** — Un repère comme [S1] ou [S2] dans une réponse. Il renvoie au fichier source d’où vient la phrase, pour que tu puisses vérifier.
- **Groupe** — Une liste nommée de fichiers (un projet, un thème) que tu peux chercher ou interroger encore et encore.
- **Gateway (passerelle)** — La petite adresse web (par défaut `http://127.0.0.1:8000`) par laquelle CartaColibri parle au moteur Colibrì.
- **Instantané (snapshot)** — Une copie HTML enregistrée d’une page web, faite à chaque sauvegarde avec web-grab, pour garder ce qu’elle disait ce jour-là.
- **Watch (surveillance)** — Deux choses : `cartacolibri watch` garde ton index à jour, et `cartacolibri webwatch` récupère à nouveau les pages que tu suis et montre ce qui a changé.

## Confidentialité et sécurité

- **Où sont rangées les choses.** Tout ce que CartaColibri écrit vit dans un seul dossier, `<home>` : par défaut `~/.cartacolibri`, ou le dossier indiqué par `CARTACOLIBRI_HOME`. Dedans : `index.sqlite` (l’index de recherche), `groups.json`, `library/uploads/` (les fichiers que tu envoies), `outputs/` (les résultats que tu as choisi d’enregistrer), `snapshots/` et `web.json` (les pages web enregistrées), `highlights.json`, `watch.json`, `settings.json` et `clip.token` (un secret pour le bookmarklet : ne le partage pas).
- **Tes originaux sont en lecture seule.** CartaColibri lit tes documents et ne les modifie, ne les déplace ni ne les supprime jamais.
- **La page est strictement locale.** Le serveur n’écoute que sur `127.0.0.1` : les autres ordinateurs de ton réseau ne peuvent pas l’atteindre. Il refuse aussi les requêtes dont l’adresse ou l’origine n’est pas locale.
- **L’en-tête `X-Colibri-UI`.** Chaque appel de la page à son propre serveur porte cet en-tête. Une page web visitée dans un autre onglet ne peut pas l’ajouter, donc elle ne peut pas faire contacter l’appli locale par ton navigateur dans ton dos.
- **Rien ne quitte `127.0.0.1`, sauf les pages que tu demandes de récupérer.** Ni télémétrie, ni statistiques, ni cloud. web-grab ne récupère que les adresses que tu lui donnes (et les pages que tu as choisi de surveiller). Le site de présentation ne charge rien depuis d’autres serveurs non plus.
- **L’historique est désactivé par défaut.** L’historique du navigateur web-grab n’existe que si tu l’actives (`cartacolibri history on`), reste sur ton disque, et `cartacolibri history clear` l’efface.

## État des lieux, sans fard

CartaColibri est un logiciel **alpha** (version 0.2.0-alpha) : il fonctionne, mais attends-toi à des aspérités et garde des sauvegardes de ce qui compte. Linux est la plateforme la mieux testée : le navigateur web-grab y est autotesté en CI. Sous **Windows et macOS**, le navigateur est compilé en CI mais n’a **jamais été lancé à la main**. Les vérifications automatiques sur GitHub (CI) **ne sont pas encore confirmées** : les jobs sont restés bloqués dans la file. Tes documents ne risquent rien dans tous les cas, puisqu’ils sont seulement lus.

## Licence et crédits

CartaColibri est publié sous **licence MIT**. C’est une implémentation Rust clean-room : elle ne parle au moteur [Colibrì](https://github.com/JustVugg/colibri) (JustVugg, Apache-2.0) que via sa passerelle HTTP publique et n’en contient aucun code. La couche d’index et de recherche suit le comportement décrit dans le manuel public de DocFetcher ; aucun code DocFetcher n’est utilisé. Le navigateur web-grab s’appuie sur la webview du système et ne contient aucun code de Falkon.

## Plus d’infos

[Documentation complète](../cartacolibri.md) · [README du moteur Colibrì](https://github.com/JustVugg/colibri) · [Contribuer](../../CONTRIBUTING.md) · [📄 Tutoriel PDF illustré](CartaColibri-Tutorial.fr.pdf)
