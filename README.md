# Ankor

**Ankor rend la couche organisationnelle d'un repo lisible et actionnable par les agents.**

Tâches, décisions d'architecture, contraintes : tout ce qui vit d'habitude dans un tracker, un wiki ou un thread — et qui n'est donc jamais accessible à un agent qui démarre sur le code. Ankor met cette information dans le repo, rattachée au code qu'elle concerne par des scopes vérifiables, dans un format qu'un agent consomme en un appel et sous 2000 tokens.

Ce qu'Ankor n'est pas : un concurrent de Linear (pas de cycles, d'estimations, de vélocité), un wiki (seul ce qui contraint ou est actionnable y entre), ni une barrière de sécurité (les garde-fous protègent contre un agent qui dérive, pas contre un acteur malveillant).

## État : pre-v1

Le CLI n'existe pas encore. Ce repo se construit en dogfoodant son propre format : le plan de développement vit dans [`.ankor/`](.ankor/), maintenu à la main en forme canonique, et validé par le parseur de référence à chaque test. Le jour où le CLI sait lire ses propres tâches, il reprend la main sans migration.

- **La spec est la source de vérité** : [`docs/ankor-spec-v1.1.md`](docs/ankor-spec-v1.1.md)
- **Le plan de développement** : `.ankor/tasks/` (DAG par `blocked_by`), les décisions dans `.ankor/adr/`

## Pour les agents

Le CLI n'étant pas disponible, lisez les fichiers directement — le format est la spec, c'est un usage de premier ordre :

1. `.ankor/adr/` — les contraintes actives sur ce que vous allez écrire. Le champ `constraint` est la règle ; le corps est le contexte.
2. `.ankor/tasks/` — le travail. Une tâche est prenable si `status: open` (ou `in_progress` sans activité récente) et si tous ses `blocked_by` sont `done`.
3. À la fin d'une tâche : faites passer les vérificateurs déclarés (`verify`, définis dans `.ankor/config.yml`), passez `status` à `done`, ajoutez une entrée de preuve et une ligne de log, incrémentez `version`.
4. Toute écriture doit rester en forme canonique : `cargo run --example check_repo` doit rester vert (round-trip octet pour octet inclus).

Les conventions détaillées pour le développement sont dans [`CLAUDE.md`](CLAUDE.md).

## Structure

    crates/ankor-core   parseur et modele de donnees — implementation de reference du format
    crates/ankor-cli    le binaire `ankor` (en construction, tache par tache)
    docs/               la spec, source de verite
    .ankor/             le plan de developpement d'Ankor, au format Ankor
    skill/              le skill d'amorcage pour les agents (embryon)

## Développement

    cargo test                          # suite complete, dont la conformite du format
    cargo run --example check_repo      # valide .ankor/ : parse, round-trip, references

Le dossier `crates/ankor-core/tests/golden/` est la suite de conformité du format, réutilisable par tout outil tiers : `valid/` doit round-tripper à l'octet près, `invalid/` doit être refusé avec l'erreur attendue.

## Licence

GPL-3.0 — voir [LICENSE](LICENSE). Le copyleft porte sur le code de l'outil, pas sur le format : vos fichiers `.ankor/` et les outils tiers qui les lisent ou les écrivent ne sont pas des œuvres dérivées.
