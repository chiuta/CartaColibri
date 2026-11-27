# Paquets de contexte

Avec le moteur propre de Colibri Companion (`corbul serve --packs DIR`), CartaColibri peut réutiliser le travail fait sur chaque source d'un appel à l'autre : résumer, puis extraire, puis comparer sur les mêmes documents sans les relire.

- Activez-le avec `--packs` (toute commande qui accepte `--url`) ou `COLI_PACKS=1`.
- Chaque source d'un prompt devient un paquet intitulé par son nom et un court #tag au lieu de son étiquette [S#] (l'étiquette dépend de la position, le même texte différerait donc d'un appel à l'autre). La fin du prompt relie chaque #tag à son étiquette [S#] : les citations et leur vérification fonctionnent comme avant.
- Tout moteur sans `/v1/packs/complete` (colibri, Ollama, services cloud) est détecté une fois et le chat ordinaire est utilisé. Les backends cloud `--provider` ne reçoivent jamais de paquets.
- La réponse est une complétion de texte brut, sans le modèle de chat : les petits modèles ajustés aux instructions peuvent les suivre moins bien qu'en chat. Vérifiez avec `--verify`.
- Pas encore mesuré sur de vrais modèles.
