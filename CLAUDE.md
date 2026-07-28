# Ankor — guide de développement pour les agents

## Contexte

Ankor est un CLI (Rust, GPL-3.0) qui rend tâches et décisions d'architecture lisibles par les agents, directement dans le repo. La spec complète est `docs/ankor-spec-v1.1.md` — c'est la source de vérité, la lire avant toute décision de design. Ce repo dogfoode son propre format : le plan de développement est dans `.ankor/`, maintenu à la main tant que le CLI ne sait pas le faire.

## Commandes

- `cargo test` — suite complète. Doit être verte avant tout commit.
- `cargo run --example check_repo` — valide `.ankor/` (parse, round-trip octet pour octet, références blocked_by). Doit être verte après toute édition de `.ankor/`.
- `cargo fmt --check` — format.

## Boucle de travail

1. Choisir une tâche dans `.ankor/tasks/` : `status: open`, tous les `blocked_by` en `done`. À priorité égale, celle qui débloque le plus de tâches, puis `created` croissant.
2. Lire les ADR dont le `scope` recouvre les fichiers visés : le champ `constraint` est contraignant.
3. Passer la tâche `in_progress`, incrémenter `version`, ajouter une ligne de log (`- <ISO-UTC> claude-code@<ctx> — message`).
4. Travailler. Le `done_criteria` est gelé : ne jamais l'éditer pour se débloquer. Une sous-tâche découverte = nouvelle tâche avec `blocked_by`, pas un critère affaibli.
5. Finir : vérificateurs de `verify` verts (définis dans `.ankor/config.yml`), `status: done`, entrée `proof` (type `commit` avec le SHA, en attendant que `ankor done` existe), log, `version` incrémentée.

**Un critère qui parle du binaire se teste par le binaire.** Quand un `done_criteria` dit « le binaire fait X », le test doit invoquer le binaire — pas seulement la fonction censée produire X. Deux défauts réels sont passés sous des tests unitaires verts : un verrou dont la libération échouait sous concurrence, et une résolution de `--repo` que le dispatch n'atteignait jamais parce qu'il rejetait le verbe avant. Dans les deux cas le code testé était juste, et le chemin réel ne l'était pas. La même règle vaut pour les plateformes : un comportement qui dépend de l'OS n'est pas vérifié tant qu'il n'a pas tourné sur les trois.

## Contraintes d'implémentation (résumé des ADR — les ADR font foi)

- Le format est la spec : `ankor-core` est l'implémentation de référence, le round-trip doit rester identique à l'octet. Tout changement de format passe par la spec d'abord, puis les goldens, puis le code.
- La surface agent est figée à 7 verbes (`context claim log done new find release`). Toute fonctionnalité nouvelle va côté humain ou côté format, jamais dans la surface agent.
- L'immuabilité est vérifiable, pas défendue : les gels sont ancrés par hash, le CLI n'est pas un gardien.
- Les claims vivent dans des refs git `refs/ankor/claims/<id>`, un par tâche, jamais dans les fichiers.
- Ankor ne commite jamais, sauf `accept`.
- Pas de nouvelle dépendance sans nécessité ; binaire statique visé ; MSRV souple mais le Cargo.lock épingle pour rustc 1.75 (levable si besoin, le noter).

## Style

- Erreurs auto-correctives : toujours la commande exacte à exécuter ensuite, jamais d'aide générique.
- Sortie terse type `git status` ; `--json` partout, opt-in strict.
- Messages, doc et commentaires sans emojis.
