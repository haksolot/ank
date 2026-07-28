# Ankor — Spécification v1.1

Statut : brouillon de travail, révision d'arbitrage
Dernière révision : 27 juillet 2026

## Arbitrages de cette révision

Points tranchés par rapport à la v1 : orientation et contraintes réconciliées (§5) · immuabilité ancrée par hash, vérifiable sans faire du CLI un gardien (§3, §8) · modèle d'exécution nominal : un worktree par agent (§7) · claims sur refs git dès le niveau 0, un ref par tâche (§7) · Ankor ne commite jamais, sauf `accept` (§12) · retour après expiration de TTL et plafond de TTL (§3) · `verify` devient une liste (§3) · `proof` devient une liste append-only, attestation CI différée en v1.1 (§3, §10) · format du log fixé : section append-only du fichier de tâche (§3) · cycle de vie de l'index fixé (§6) · timeout des vérificateurs fixé (§4) · `--reason` obligatoire sur `release` (§4) · signaux `check` étendus (§4) · identité, rôles par défaut et vérification de signature précisés (§8).

Tous les points ouverts de la v1 sont tranchés : licence GPL-3.0, Windows natif en v1, driver de merge et attestation CI spécifiés mais implémentés en v1.1 (§13).

Ajouts de la révision c : code de sortie 9 (environnement de vérification défaillant, distinct d'un échec de tâche) · budget de contexte chiffré et seuil mécanique du sur-contraint (§5) · hash des contraintes applicables dans le claim, avertissement de `done` si le contexte a changé en cours de tâche (§7) · une entrée de preuve par vérificateur, `verify` vide équivaut à absent (§4) · `claim` refuse un bloqueur `closed`, `close` révoque le claim actif (§3) · `created` en UTC + signal de plausibilité (§3, §4) · mécanique de ré-acquisition explicitée (§3) · signal d'accaparement par logs vides (§4) · hash chaîné du log envisagé et écarté, avec la raison (§3).

Ajouts de la révision b, issus de relecture externe : champ `created` au socle commun (ordre déterministe et signal de création en rafale) · statut `closed` et commande humaine `close --reason` pour l'abandon ratifié (§3, §4) · hash de la définition du vérificateur ancré dans la preuve (§4) · risque d'inondation de tâches explicitement accepté et motivé (§3) · `review` filtre par scopes vivants (§4).

---

## 1. Intention

Ankor rend la **couche organisationnelle d'un repo lisible et actionnable par les agents**.

Tâches, décisions d'architecture, arbitrages : tout ce qui vit aujourd'hui dans un tracker, un wiki ou un thread, et qui n'est donc jamais accessible à un agent qui spawn sur le code. Ankor met cette information dans le repo, rattachée au code qu'elle concerne, dans un format qu'un agent consomme sans effort et sans coût de tokens excessif.

### Non-objectifs

Ces exclusions sont la spec, pas des omissions.

- **Ce n'est pas un concurrent de Linear.** Pas de cycles, d'estimations, de vélocité, de roadmap, de burndown. Ankor peut exporter vers un tracker pour la visibilité humaine, il ne le remplace pas.
- **Ce n'est pas un wiki.** N'entre dans Ankor que ce qui est *actionnable ou contraignant pour un agent*. Une décision qui contraint du code : oui. Un compte-rendu de réunion : non.
- **Ce n'est pas une barrière de sécurité.** Les permissions protègent contre un agent qui déraille, pas contre un acteur malveillant.

### Critères de succès

1. Un agent qui spawn sur un chemin obtient tout ce qui le contraint en un seul appel, sous 2000 tokens.
2. Un agent ne peut pas se déclarer « fini » sans preuve.
3. Un agent ne peut pas affaiblir une contrainte pour se débloquer — et quand il fixe lui-même sa barre, c'est visible.
4. L'outil fonctionne en solo local sans aucune configuration ni service.

---

## 2. Principes de design

| Principe | Conséquence concrète |
|---|---|
| Le format est la spec | Le CLI est une implémentation de référence, pas un gardien. Tout outil peut lire/écrire. |
| L'immuabilité est vérifiable, pas défendue | Le CLI ne peut pas empêcher une édition directe ; chaque gel est donc ancré par un hash dans un artefact que l'éditeur ne contrôle pas, et `check` compare. |
| Shell plutôt que MCP | Dénominateur commun entre agents. Pas de sérialisation protocolaire par appel. |
| Ancrer, ne pas faire confiance | Toute transition d'état exige une preuve externe vérifiable. |
| Immuable par défaut | On ne modifie pas une décision, on la remplace. L'affaiblissement devient visible. |
| Terse par défaut | Sortie type `git status`. Le JSON est opt-in strict. |
| Dégradation, pas échec | Sans remote, sans daemon : Ankor fonctionne toujours, en mode réduit. La dégradation porte sur les services et le réseau, pas sur le substrat — git est une dépendance dure (§7). |

---

## 3. Modèle de données

### Deux plans orthogonaux, pas une pyramide

Ankor ne modélise pas une chaîne décision → épique → tâche. Les contraintes et le travail sont **deux plans indépendants, joints uniquement par le scope**.

C'est la propriété qui distingue Ankor d'un tracker : un agent reçoit ce qui le contraint sans traverser aucune hiérarchie, et une contrainte s'applique à du travail qui n'existait pas quand elle a été écrite. Une pyramide imposerait de rattacher chaque tâche à une décision parente — inexact dans les faits, et coûteux en traversée au moment de construire le contexte.

### Où se loge la précision

Ankor permet de planifier avec précision, mais la précision porte sur **la spécification du travail**, pas sur son ordonnancement calendaire. Quatre champs la portent : `scope` (où), `done_criteria` (ce qui prouve que c'est fini), `constraint` (sous quelles règles), `blocked_by` (dans quel ordre).

Ce qui est délibérément absent : estimations, points, priorités déclarées, cycles, échéances. Ce sont des instruments de coordination d'équipes humaines dans le temps. Ils n'aident pas un agent à travailler correctement, et ils sont la pente qui mène tout droit à réimplémenter Linear.

### Le regroupement se fait par scope

Il n'existe ni épique, ni jalon, ni étiquette. « Tout ce qui concerne la migration auth » se répond par `ankor context src/auth/`.

Le scope est un axe de regroupement supérieur à une étiquette pour cette raison précise : il est **vérifiable**. Une étiquette est déclarative, elle dérive et personne ne la nettoie ; un glob se confronte au système de fichiers. C'est aussi ce qui garantit qu'un regroupement ne se périme pas silencieusement quand le code bouge.

### Socle commun

Tout objet Ankor est un fichier markdown avec frontmatter YAML. Champs communs :

| Champ | Rôle |
|---|---|
| `id` | Identifiant canonique, immuable, généré sans coordination |
| `type` | `task` \| `adr` |
| `slug` | Cosmétique, jamais utilisé pour la résolution |
| `created` | Horodatage ISO 8601 de l'acte de création, **toujours en UTC** (suffixe `Z`) : l'ordre du §5 ne doit pas dépendre d'un fuseau. Immuable. C'est lui qui rend l'ordre des tâches déterministe sans dépendre de git, et qui donne à `check` une base pour les signaux de création en rafale et de plausibilité. |
| `scope` | Liste de globs. Source de vérité pour le routage du contexte. **Obligatoire** : sans scope une entité n'apparaît dans aucun `context` et devient invisible. `new` échoue plutôt que de créer un orphelin silencieux. |
| `status` | Cycle de vie typé par entité |
| `version` | Entier, incrémenté à chaque écriture. Compare-and-swap intra-arbre (§7). |

### Allocation des identifiants

Emprunt direct à git, avec une adaptation.

Git dérive l'ID du contenu, ce qui suppose l'immuabilité. Une tâche mute (statut, log, titre) : un ID dérivé du contenu changerait à chaque édition et casserait toute référence déjà écrite. Ankor hashe donc **l'acte de création** — timestamp, identité de l'agent, titre initial, aléa — qui est immuable. L'ID est stable à vie et généré sans coordination, ce qui est indispensable en offline-first.

- **12 caractères hexadécimaux** stockés. En dessous de 8, collision d'anniversaire au premier millier d'entités.
- **Préfixe court accepté** en entrée et affiché en sortie (`TASK-8f3a`).
- **L'ambiguïté est une erreur.** Un préfixe qui matche deux entités échoue avec la liste des candidats. L'outil ne devine jamais.

### Tâche

`.ankor/tasks/TASK-8f3a91c2d4e7.md`

```yaml
---
id: TASK-8f3a91c2d4e7
type: task
slug: migrer-auth-sessions
title: Migrer l'auth vers des sessions opaques
created: 2026-07-25T09:14:00Z
status: in_progress          # open | in_progress | done | closed   (blocked est dérivé)
scope:
  - src/auth/**
  - src/middleware/session.ts
blocked_by: [TASK-51c2a7f0]  # DAG. Vide = prête à être claim.
done_criteria: |             # requis pour claim, gelé par hash ensuite
  Les tests d'intégration auth passent, et plus aucune
  référence à jwt.verify dans src/auth/
criteria_by: creator         # creator | claimer — posé par l'outil, signal pour check
verify: [auth-tests, no-jwt] # liste de vérificateurs de config.yml, tous doivent passer
proof:                       # liste append-only, requis pour done
  - type: test               # test | commit | human-review | assertion
    ref: local/9c1f4a@a3f9c21
    tree: scope/4be2d10c     # hash du contenu des fichiers du scope à l'exécution
    verifier: auth-tests@1f2e3d4c   # hash de la définition exécutée (§4)
  - type: test
    ref: local/e51b22@a3f9c21
    tree: scope/4be2d10c
    verifier: no-jwt@9ab0c1d2
schema: 1
version: 7
---

Contexte libre, notes, liens.

## Log
- 2026-07-26T14:02Z claude-code@host-3 — jwt.verify supprimé de session.ts
- 2026-07-26T14:31Z claude-code@host-3 — released: nécessite un accès au store Redis de staging
```

**Le format du log est fixé** (ex-point ouvert 3) : une section `## Log` en fin de fichier de tâche, append-only, une ligne horodatée par entrée. Appendre en fin de fichier produit un diff git d'une ligne, ce qui préserve la propriété de récupération (§12). Un fichier séparé aurait donné des diffs équivalents en doublant le nombre d'objets à résoudre ; le fichier unique garde le principe « le format est la spec » simple pour les outils tiers. `log` écrit ici et incrémente `version`. Le log est une **trace de travail, pas une preuve** : rien d'autoritaire n'y est ancré — les gels et les preuves ont leurs ancres par hash propres — et une entrée passée réécrite est un diff git visible en revue comme n'importe quelle falsification d'historique. C'est pourquoi un hash chaîné du log, envisagé, a été écarté : il alourdirait le format pour défendre une surface qui ne porte aucune autorité.

**Le claim n'est pas dans le fichier.** Il vit exclusivement dans le plan de coordination éphémère (§7). L'inscrire ici produirait un diff git à chaque prise de tâche, ce que la séparation des deux plans existe précisément pour éviter. Une tâche `in_progress` sans claim actif est simplement une tâche dont le TTL a expiré : elle est reprenable, et son log dit où le précédent s'est arrêté.

**`schema`** porte la version de format, avec migration explicite et jamais de rupture silencieuse. C'est la contrepartie de la promesse « le format est la spec » : un outil tiers doit pouvoir refuser proprement un fichier qu'il ne sait pas lire.

**Cycle de vie.** `open` → `in_progress` (via `claim`) → `done` (via `done`, preuve obligatoire). Il n'y a pas de statut `claimed` distinct : un claim réussi met directement en travail. **`claim` sur une tâche `in_progress` sans claim actif est une transition légale** — c'est la reprise après expiration, pas une anomalie. Après `done`, la seule écriture légale est l'**ajout** d'une preuve à la liste `proof` ; toute autre modification est remontée par `check`.

**`closed` est l'abandon ratifié.** Terminal, accessible depuis `open` et `in_progress`, réservé aux identités humaines via `ankor close <id> --reason <r>` — la raison part dans le log. C'est la réponse au vieillissement d'un corpus actif : les scopes morts et tâches orphelines que `check` détecte doivent pouvoir être fermés explicitement, jamais automatiquement, et jamais par suppression de fichier (supprimer casserait les références `blocked_by` des autres tâches, alors que `closed` les préserve). **`closed` ne débloque pas** : une tâche fermée n'a pas été faite, donc ses dépendantes restent bloquées, et `check` remonte « bloqué par une tâche fermée » pour que l'humain tranche — refermer en cascade, ou réécrire la dépendance. Deux précisions opérationnelles : `claim` refuse une tâche dont un `blocked_by` est `closed` (code 7, en nommant le bloqueur fermé, comme pour tout bloqueur actif), et `close` sur une tâche `in_progress` révoque le claim actif dans la même opération — le ref est supprimé, l'agent titulaire l'apprend à son prochain `log` (code 6, la tâche n'est plus en travail).

**`blocked` n'est pas un statut, c'est une propriété dérivée.** Une tâche est bloquée si et seulement si elle a au moins un `blocked_by` non terminé. Rien n'est saisi à la main, donc rien ne peut devenir périmé. `claim` refuse une tâche bloquée et nomme le bloqueur.

**`blocked_by` est la seule relation entre tâches.** Un DAG, pas un arbre : ni parent/enfant, ni rollup, ni cascade. Trois raisons, dans l'ordre d'importance.

*Le rollup est une complétion sans preuve.* « Le parent est fini quand les enfants sont finis » est structurellement la même faille que `assertion:`, dissimulée dans la topologie au lieu d'être écrite dans un champ. Avec un DAG, quand les bloqueurs sont terminés la tâche est **débloquée, pas terminée** : elle passe encore son propre `done` avec sa propre preuve. Le parent vérifie le tout, pas la somme des parties — et c'est exactement là que se logent les régressions d'intégration.

*La décomposition est découverte, pas planifiée.* Un humain casse un epic de haut en bas. Un agent découvre en cours de route que sa tâche en exige une autre. C'est un ordonnancement, pas une containment : forcer un arbre obligerait à décider d'une parenté au moment où l'on ne connaît qu'un ordre.

*Un même travail sert souvent deux tâches.* « Ajouter l'adaptateur Redis » bloque à la fois la migration auth et le nettoyage des sessions. Un arbre l'interdit, un DAG le représente sans duplication.

Les cycles sont refusés à l'écriture et remontés par `check`.

**Bloquer diffère l'obligation, ne la libère pas.** Un agent peut créer des bloqueurs après avoir claim, ce qui est le cas légitime des sous-tâches découvertes — mais c'est aussi une porte de sortie possible pour un agent en difficulté. Le garde-fou n'est pas d'interdire l'acte mais d'en supprimer le gain : le `done_criteria` reste gelé, la tâche reste à terminer, et créer un bloqueur coûte le prix d'une vraie tâche (scope et critère vérifiable obligatoires), donc un bloqueur fictif est visible à l'œil nu. `check` remonte le motif « bloqueurs créés par le même agent après claim » comme signal, non comme faute. Fausser doit coûter plus cher que faire.

**`done_criteria` est requis pour `claim`, et son gel est ancré par hash.** Le gel ne peut pas intervenir plus tard : une tâche créée sans critère deviendrait alors définitivement incritérisable une fois claim. `claim` échoue donc si le champ est vide, avec la commande exacte pour le poser et claim dans le même appel.

Le mécanisme du gel tient compte du fait que le CLI n'est pas un gardien : n'importe quel outil peut réécrire le fichier. Le gel est donc **vérifiable, pas défendu** : `claim` enregistre le hash du `done_criteria` dans l'enregistrement de claim (§7), `done` vérifie que le critère courant correspond à ce hash avant d'exécuter quoi que ce soit, et inscrit le hash dans la preuve. Un critère modifié entre claim et done fait échouer `done` avec le code 6, et `check` remonte le cas. Éditer le fichier ne débloque rien.

Le gel empêche d'affaiblir un critère *après coup* ; il n'empêche pas de poser un critère complaisant *au moment du claim*. C'est pourquoi le champ `criteria_by` trace qui a posé le critère : une tâche créée par un humain porte son critère dès la création (`creator`), et `check` remonte « critère posé par le claimer » comme signal — même logique que les bloqueurs auto-créés, visible sans être interdit.

**TTL du claim.** Court, 30 minutes par défaut, **plafonné par `claim_ttl_max` dans `config.yml`** (2 heures par défaut) — un agent ne peut pas s'accorder 24 heures et accaparer. Il est **renouvelé implicitement par `log`** : travailler suffit à garder le verrou, il n'y a pas de verbe `heartbeat` à mémoriser.

**Retour après expiration.** Un build de 40 minutes sans `log` fait expirer le claim ; c'est un cas normal, pas une faute. À l'expiration, la tâche reste `in_progress` et redevient réclamable. Quand le titulaire initial revient : si personne n'a repris la tâche, `log` et `done` **ré-acquièrent silencieusement** le claim et continuent ; si un autre agent l'a reprise entre-temps, ils échouent avec le code 4 et le nom du nouveau titulaire. Mécaniquement, « silencieusement » signifie : vérifier l'absence de claim actif pour la tâche — le ref `refs/ankor/claims/<id>` — puis le recréer au nom de l'agent courant, les deux étapes reposant sur la primitive atomique de la mise à jour de ref. Aucune donnée n'est perdue dans les deux cas — le log dit où chacun s'est arrêté.

### ADR

`.ankor/adr/ADR-3c7e0b9142af.md`

```yaml
---
id: ADR-3c7e0b9142af
type: adr
slug: sessions-opaques
title: Sessions opaques plutôt que JWT stateless
created: 2026-07-18T11:02:00Z
status: accepted             # proposed | accepted | superseded
scope:
  - src/auth/**
constraint: |                # le seul champ injecté dans le contexte
  Ne pas introduire de JWT auto-porteur pour l'auth utilisateur.
  Toute session passe par le store Redis.
see: src/auth/session_store.ts    # optionnel, pour les contraintes positives
supersedes: ADR-9a12ff03b8e1
ratified: 4c1e9a20            # commit signé de ratification (posé par accept)
version: 2
---

Décision, alternatives écartées, conséquences.
```

**`constraint` est court et impératif.** C'est lui seul qui part dans le contexte de l'agent, jamais le corps du document. Un ADR de trois pages coûte ainsi une trentaine de tokens à l'injection.

**`see`** répond au fait qu'une contrainte négative (« ne pas faire X ») se respecte sans contexte, alors qu'une contrainte positive (« tout passe par l'adaptateur X ») a besoin d'un pointeur vers le code de référence.

**Immuabilité, ancrée comme celle des tâches.** Un ADR `accepted` a `constraint`, `scope` et `status` verrouillés ; le corps reste éditable. Le verrou est vérifiable : le commit signé de ratification (§8) enregistre le hash de `constraint` et de `scope` au moment de l'acceptation, et `check` compare l'état courant à ce hash. Un ADR dont la contrainte a divergé du hash ratifié est remonté comme **altéré** — et son injection dans `context` est suspendue avec un avertissement explicite, car injecter une contrainte altérée reviendrait à laisser l'éditeur réécrire la règle.

Modifier une décision = créer un nouvel ADR qui la `supersedes`. **La transition `accepted` → `superseded` est la seule écriture légale sur un ADR accepté**, et elle est effectuée par `accept` du nouvel ADR, dans le même commit de ratification — le remplacement et son autorisation sont indissociables.

**Ratification.** Un agent crée en `proposed`. Seule une identité humaine promeut en `accepted`. Un ADR `proposed` est visible en mode orientation, **jamais injecté en mode exécution** : non contraignant signifie qu'il ne doit pas consommer le budget d'attention d'un agent en train de coder.

Le principe sous-jacent, qui explique l'asymétrie avec les tâches : **la ratification s'applique là où un artefact engage les autres, pas là où il enregistre du travail.** Un ADR contraint tout agent qui passera après lui ; une tâche n'engage personne. D'où `new task` sans restriction et `new adr` en `proposed`.

Le risque symétrique — un agent qui déraille et inonde le repo de tâches — est **accepté, sans quota**. Un quota serait inapplicable dans ce design : le format est la spec, un agent écrit les fichiers directement, et il n'existe aucun arbitre central en offline-first pour compter. La défense est la visibilité, pas la restriction : chaque tâche coûte un scope valide, `check` remonte la création en rafale par une même identité (via `created`), et `review` présente les créations par auteur. L'inondation est un diff bruyant en revue, pas un état silencieux.

---

## 4. Surface CLI

Le budget de mémorisation est **par audience**, pas global. Git compte plus de cent commandes et reste apprenable parce que personne n'a jamais besoin des cent. Ankor applique la même séparation : une surface agent, figée et minimale, et une surface humaine qui peut s'enrichir sans jamais la toucher.

### Surface agent — sept verbes, figés

```
Boucle :      context → claim → log → done
Hors-boucle : new, find, release
```

C'est le seul contenu du SKILL.md. Il ne doit plus jamais grossir : toute fonctionnalité nouvelle atterrit côté humain ou côté format. Un agent qui veut le corps complet d'une entité lit le fichier — le format est la spec, `cat` est déjà le `show` des agents.

### Surface humaine

```
review    file d'attente de ratification, propositions en attente, santé du corpus
accept    promeut un ADR proposed -> accepted (produit le commit signé, §8, §12)
check     invariants mécaniques, code de sortie exploitable en CI
show      affichage complet d'une entité, corps inclus
close     ferme une tâche qui ne sera jamais faite (--reason obligatoire)
```

`review` et `accept` ne sont pas du confort : **le modèle d'autorité des ADR en dépend entièrement**. Sans eux, un agent crée en `proposed` et rien ne devient jamais contraignant.

`review` filtre par défaut sur les **scopes vivants** ; les entités à scope mort sont regroupées en une section de nettoyage avec la commande `close` en suggestion. Un corpus qui vieillit produit ainsi une file de fermeture explicite, pas du bruit diffus.

Tout le reste (édition de champs, réordonnancement, suppression) passe par l'édition directe du fichier, puisque le format est la spec.

### HEAD

Emprunt à git, et probablement le plus rentable. `claim` pose un pointeur « tâche courante » par agent. Les commandes suivantes s'en passent d'ID :

```
$ ankor claim 8f3a
claimed TASK-8f3a migrer-auth-sessions -> HEAD

$ ankor log "jwt.verify supprime de session.ts"
$ ankor done
```

L'agent ne peut pas se tromper d'identifiant, et chaque itération économise un aller-retour de contexte.

**HEAD n'est pas stocké, il est dérivé** : c'est la tâche sur laquelle l'agent courant détient un claim actif. Rien à synchroniser, rien à nettoyer, aucun état qui puisse devenir incohérent avec le claim réel.

Cela suppose et impose **un claim actif à la fois par agent**. C'est une contrainte utile en soi : un agent doit terminer ou relâcher avant de passer à autre chose, ce qui empêche l'accaparement de tâches et garde le travail en cours lisible pour les autres.

L'ID optionnel de `log`, `done` et `release` est donc toujours redondant : il n'existe que pour l'explicite en script, et **doit correspondre à HEAD**, sinon erreur 6. Il n'est jamais un moyen d'agir sur la tâche d'un autre.

### Reprise et abandon

`release` ferme un trou réel : un agent qui constate qu'il ne peut pas faire la tâche n'a sinon que l'expiration du TTL, soit trente minutes de travail mort pour les autres.

```
$ ankor release --reason "necessite un acces au store Redis de staging"
released TASK-8f3a -> open
```

**`--reason` est obligatoire.** `release` est le mécanisme de délégation entre agents : la raison part dans le log, et l'agent suivant qui claim la tâche reçoit le log récent dans son `context`, donc il reprend là où le précédent s'est arrêté plutôt que de repartir de zéro. Un release muet est exactement le trou que ce verbe existe pour fermer — l'outil le refuse avec la commande complète en exemple.

### Commandes

```
ankor context [<path>]        [--json] [--limit N]
ankor claim <id>              [--criteria <c>] [--ttl 30m]
ankor log [<id>] <message>
ankor done [<id>]             [--proof <type>:<ref>]
ankor release [<id>] --reason <r>
ankor new task --title <t> --scope <glob>... [--criteria <c>] [--blocked-by <id>...]
ankor new adr  --title <t> --scope <glob>... --constraint <c>
ankor find <query>            [--type task|adr] [--status ...] [--scope <path>]
ankor review [<path>]
ankor accept <id>
ankor close <id> --reason <r>
ankor check [<path>]
ankor show <id>
```

**`ankor context` sans argument** couvre tout le repo. C'est le premier appel qu'un agent doit faire, avant même de savoir sur quel chemin il travaille — un agent lancé sur « corrige le bug de login » ne connaît pas encore son périmètre.

**`context` avec un claim actif est toujours en mode exécution** (§5). Un argument de chemin est alors ignoré, avec une ligne d'avertissement : `claim actif sur TASK-8f3a, contexte d'execution (release pour explorer ailleurs)`. Explorer un autre périmètre en cours de tâche est précisément ce que la règle un-claim-par-agent décourage.

`--limit` ne s'applique **qu'aux tâches**, jamais aux contraintes.

**`find` est soumis au même plafond que `context`**, une ligne par résultat, et annonce ce qu'il a coupé. Une commande de recherche sans budget est un vecteur d'explosion de contexte au moins aussi efficace qu'un `context` mal borné. `--scope <path>` filtre par correspondance de scope — c'est la commande vers laquelle pointent les compteurs de troncature (§5).

**`log` exige de détenir le claim.** C'est le registre d'ancrage de la tâche : si n'importe qui peut y écrire, il cesse d'être une trace fiable de ce qu'a fait le titulaire. Un humain qui veut annoter édite le corps du fichier — ce qui est déjà la voie normale pour tout ce qui n'est pas une transition d'état.

Les commandes humaines prennent des **chemins**, uniformément : `review [<path>]` et `check [<path>]` partagent la même sémantique de périmètre que `context`.

Flags globaux, volontairement limités à trois : `--json`, `--quiet`, `--repo <path>`. Chaque flag global est un coût de mémorisation. `--json` est disponible sur toutes les commandes sans exception : la scriptabilité intégrale est un invariant, pas une option.

### Codes de sortie

La sémantique est portée par le code pour que le shell route sans parser la sortie.

| Code | Signification |
|---|---|
| 0 | ok |
| 1 | erreur générique |
| 2 | entité introuvable ou préfixe ambigu |
| 3 | conflit de version — relire et réessayer |
| 4 | claim tenu par un autre agent |
| 5 | preuve manquante ou invalide |
| 6 | transition illégale, ou champ gelé modifié (hash divergent) |
| 7 | prérequis manquant — critère absent, ou tâche bloquée |
| 8 | `check` : invariants violés (réservé à `check`, pour la CI) |
| 9 | environnement de vérification indisponible — pas un échec de la tâche |

Les codes 3 et 4 sont ceux que la boucle agentique doit savoir gérer. Le 3 signifie littéralement : « refais `context`, quelqu'un a bougé ». `check` sort en 0 quand le corpus est sain, en 8 quand il a des findings — jamais en 1, pour que la CI distingue un corpus malade d'un outil cassé.

### Erreurs auto-correctives

Jamais d'aide générique, toujours la commande exacte à exécuter ensuite. Un aller-retour d'erreur bien conçu coûte moins cher que trois tentatives à l'aveugle.

```
$ ankor done
error[5]: preuve requise pour passer TASK-8f3a a done
  done_criteria: "Les tests d'integration auth passent, et plus
                  aucune reference a jwt.verify dans src/auth/"
  -> ankor done --proof test:<ref-du-run-ci>
```

```
$ ankor claim 8f3a
error[4]: TASK-8f3a tenue par codex@host-9 (expire dans 12m)
  -> ankor claim 51c2   (autre tache prete sur ce scope)
```

```
$ ankor claim 51c2
error[7]: TASK-51c2 n'a pas de done_criteria
  -> ankor claim 51c2 --criteria "<critere verifiable>"
```

### Preuves

**Ankor exécute lui-même la vérification.** C'est le point central : si l'agent lance les tests puis rapporte le résultat via `--proof`, rien n'est ancré — il peut affirmer que ça passe. La tâche déclare ses vérificateurs, `ankor done` les lance, capture les codes de sortie et un hash des sorties. L'agent ne s'auto-rapporte jamais.

```
$ ankor done
verifying done_criteria hash ... ok
running: auth-tests ... ok (2.4s)
running: no-jwt ... ok (0.1s)
proof recorded: auth-tests -> local/9c1f4a@a3f9c21
proof recorded: no-jwt -> local/e51b22@a3f9c21  (tree:scope/4be2d10c)
```

**Deux modes, jamais ambigus.** Si la tâche déclare un `verify`, `ankor done` exécute **tous** les vérificateurs de la liste — un `done_criteria` composite (« les tests passent *et* plus de jwt.verify ») se mécanise par plusieurs vérificateurs, pas par un seul qui n'en couvre qu'une partie — et `--proof` est refusé : l'agent ne peut pas court-circuiter. Chaque vérificateur exécuté produit **sa propre entrée de preuve**, avec son hash de sortie et sa définition ancrée. Sans `verify` — champ absent ou liste vide, les deux formes sont équivalentes et la forme canonique omet le champ vide —, `--proof` est obligatoire et Ankor valide ce qu'il peut : `commit:` est vérifié avec git, `human-review:` et `assertion:` sont enregistrés tels quels et marqués comme non vérifiés.

Un vérificateur en échec ou en timeout, la transition est refusée. Aucune dépendance à un service : c'est utilisable **dans** la boucle, pas seulement à la fin.

### Vérificateurs nommés

Le champ `verify` d'une tâche référence des vérificateurs déclarés dans `config.yml`, jamais une commande shell inline.

```yaml
verifiers:
  auth-tests:
    run: pytest tests/auth/ -q
    timeout: 10m              # defaut : 10m, l'echec de timeout = code 5
  no-jwt:
    run: "! grep -rq jwt.verify src/auth/"
```

Raison : une tâche peut arriver par une PR depuis un fork. Une commande inline serait de l'exécution de code arbitraire déclenchée par `ankor done`. Git a exactement ce problème avec les hooks et l'a résolu en ne les exécutant jamais au clone. Ici, les vérificateurs vivent dans un fichier contrôlé par le repo, donc leur modification passe par la revue de code comme n'importe quel changement.

**Exécution : toujours `sh -c`, sur les trois OS.** Linux, macOS et Windows sont supportés nativement en v1. Sur Windows, `sh` est résolu depuis Git for Windows, qui l'embarque — Ankor exige déjà git, la dépendance est donc gratuite et les vérificateurs s'écrivent une seule fois, en syntaxe POSIX, pour toute l'équipe. Un `sh` introuvable est une erreur explicite avec le lien d'installation, jamais un repli silencieux vers `cmd`. **Un environnement défaillant n'est pas un échec de la tâche** : `sh` introuvable, commande absente (code shell 126/127), impossibilité de lancer le processus sortent en code 9 avec la commande exacte qui a échoué — l'agent doit signaler ou réparer l'environnement, pas conclure que son code est faux. Le code 5 reste réservé au vérificateur qui a tourné et dit non. Le **timeout est fixé** (ex-point ouvert 4) : 10 minutes par défaut, surchargeable par vérificateur, dépassement = échec code 5 avec le temps écoulé dans le message.

`config.yml` reste éditable par un agent — remplacer un vérificateur par `true` est le contournement évident. Deux défenses complémentaires, aucune ne reposant sur la bonne foi. D'abord, **la preuve enregistre le hash de la définition exécutée** (`verifier: auth-tests@<hash>`, hash normalisé de l'entrée `run` + `timeout`) : ce qui a réellement tourné est ancré dans la preuve, pas dans l'état courant de `config.yml`, donc un vérificateur affaibli avant ou après le `done` — dans le même commit ou dans un autre — est détectable en comparant le hash de la preuve à la définition du commit correspondant. Ensuite, `check` remonte les motifs : **vérificateur modifié dans la fenêtre d'activité de la tâche** (entre la première entrée de log et le `done`), et **hash de preuve divergent de la définition en vigueur au commit du `done`**. Fractionner le contournement en plusieurs commits ne le cache plus, il reste un diff en revue — fidèle au principe « fausser coûte plus cher que faire ».

### Hiérarchie de confiance

| Type | Ce qui est garanti |
|---|---|
| `assertion:"..."` | Rien. L'agent affirme. **Marqué faible** dans `check`. |
| `test:local/<hash>@<sha>` | Ankor a exécuté, dans un environnement que l'agent contrôle |
| `commit:<sha>` | Vérifiable par n'importe qui avec `git` |
| `test:ci://<ref>` | Environnement tiers, hors de portée de l'agent |

La ligne de partage n'est pas local contre hébergé, c'est **qui contrôle l'environnement**. En local, un agent peut affaiblir un test pour le faire passer — même classe de problème qu'un ADR édité pour se débloquer.

**Ce que la preuve locale ancre.** Le cas nominal d'un agent est un arbre de travail non commité : ancrer la preuve sur le SHA de HEAD seul pointerait presque toujours un état périmé. La preuve enregistre donc trois choses : le SHA de HEAD, un indicateur d'arbre sale, et **un hash du contenu des fichiers du scope au moment de l'exécution** (`tree:scope/<hash>`, à la git hash-object). C'est ce dernier qui capture réellement ce qui a été testé. `check` remonte par ailleurs le cas où la tâche a elle-même modifié les fichiers de test qu'elle invoque.

Les niveaux se cumulent : preuve locale au moment du `done`, référence CI **ajoutée** plus tard à la liste `proof` — l'ajout de preuve est la seule écriture légale post-`done` (§3). Le mécanisme d'attestation CI lui-même (`ankor attest`) est différé en v1.1 (§10) : la structure de données le permet dès maintenant, la commande viendra quand une CI l'appellera.

Le type `assertion` existe parce que « refactor pour lisibilité » n'a pas de hash à attacher. Il est autorisé mais visible comme faible, ce qui évite qu'un `--force` devienne le chemin par défaut en deux semaines.

### Périmètre de `check`

Récapitulatif des invariants et signaux, tous mécaniques :

- claims expirés, cycles de `blocked_by`, chaînes de supersede rompues, scopes morts (aucun fichier matché), scopes sur-contraints (§5) ;
- champs gelés divergents de leur hash d'ancrage — `done_criteria` vs claim, `constraint`/`scope` vs commit de ratification ;
- preuves faibles (`assertion`, non vérifiées), tâches `done` modifiées au-delà d'un ajout de preuve ;
- signaux comportementaux, remontés sans être des fautes : bloqueurs créés par le titulaire après claim, critère posé par le claimer, vérificateur modifié dans la fenêtre d'activité de la tâche ou hash de preuve divergent de sa définition, tests du scope modifiés par la tâche qui les invoque, création en rafale par une même identité (volume anormal de `new` sur une fenêtre courte, via `created`), `created` implausible (dans le futur, ou nettement antérieur au commit qui introduit le fichier — le champ est déclaratif, git est l'ancre), renouvellements de claim répétés sans modification des fichiers du scope (accaparement possible ; signal best-effort, l'arbre d'un autre agent n'est pas observable), contrainte acceptée postérieurement au claim d'une tâche en cours, tâches bloquées par une tâche `closed` ;
- marqueurs de conflit git non résolus dans des fichiers `.ankor/` (§7).

---

## 5. Budget d'attention

Le point le plus déterminant pour l'usage réel. Un `context` qui explose sur un gros repo est un `context` que l'agent finira par ignorer.

### Deux moments, deux sorties

`context` sert deux situations que l'on traitait à tort comme une seule.

**Avant claim — orientation.** L'agent ne sait pas encore quoi faire. Largeur, pas de profondeur : les contraintes actives du périmètre au format compact (id + texte de `constraint`, jamais le corps), les propositions non contraignantes en une ligne, et les tâches ouvertes en une ligne chacune. **Pas de `done_criteria`, pas de log** — il ne code pas encore, le détail d'exécution serait du bruit. Les contraintes, elles, sont présentes dès l'orientation : choisir une tâche en connaissant les règles du périmètre est exactement ce que l'orientation sert.

**Après claim — exécution.** HEAD est posé. Inversion : plus aucune autre tâche, mais le `done_criteria` complet, les contraintes qui matchent le scope **de la tâche**, et les dernières entrées de log.

Même commande, sortie pilotée par HEAD. Rien de plus à mémoriser pour l'agent, et l'essentiel du contexte inutile disparaît.

```
$ ankor context src/auth/

CONSTRAINTS (2 active)
  ADR-3c7e  Ne pas introduire de JWT auto-porteur pour l'auth
            utilisateur. Toute session passe par le store Redis.
  ADR-8b41  Rate limiting obligatoire sur tout endpoint public.

PROPOSED (1, non-binding)
  ADR-19d0  [pi@host-2] Preferer les migrations idempotentes

TASKS (2)
  TASK-8f3a  [claimed:claude-code@host-3] Migrer l'auth vers sessions opaques
  TASK-51c2  [open] Ajouter la rotation des secrets

> ankor claim 51c2 to start
```

### Priorité de troncature

**En mode exécution, une contrainte n'est jamais tronquée.** Couper une contrainte contraignante signifie qu'un agent peut violer une règle qu'il n'a jamais vue — un `+12 autres` discret serait le pire comportement possible. Le design biphasé rend la garantie tenable : après claim, le périmètre est celui de la tâche seule, donc peu de contraintes matchent. Le budget est concret : `context_budget` dans `config.yml`, mesuré en caractères, 8000 par défaut (de l'ordre de 2000 tokens — le caractère est la seule unité mesurable sans dépendre d'un tokenizer). Un scope est **sur-contraint** quand les contraintes seules en consomment plus de la moitié en mode exécution : seuil mécanique, implémentable tel quel, et `check` le remonte comme tel — c'est un problème de corpus, pas d'affichage.

### Ordre des tâches

Un agent face à huit tâches prêtes doit en choisir une sans hésiter et sans inventer un critère. L'ordre est donc **déterministe et dérivé**, jamais déclaré :

1. Nombre de tâches que celle-ci débloque directement, décroissant
2. À égalité, champ `created` croissant (déterministe sans dépendre de git)

Les tâches sur le chemin critique remontent naturellement, sans champ `priority` à maintenir ni à faire dériver. Un humain qui veut orienter le travail le fait en créant ou en claim, pas en réordonnant une liste.

### Fin de boucle

Aucune tâche prête sur le périmètre est un état normal, pas une erreur. `context` le dit explicitement et sort en 0 :

```
no ready tasks in scope (3 blocked, 1 in progress by codex@host-9)
```

Un agent en boucle a besoin d'un signal d'arrêt net. Une sortie vide se lit comme une panne et déclenche des reprises inutiles.

En mode orientation, où l'agent ne code pas encore, la troncature est acceptable. L'ordre de coupe :

1. Les tâches d'abord, avant toute contrainte
2. Contraintes au scope le plus **spécifique** conservées — un glob étroit bat `src/**`, il a été écrit pour ce code précis
3. Contraintes dont le vocabulaire recoupe les titres des tâches du périmètre
4. Le reste en compteur : `+12 contraintes larges, ankor find --type adr --scope <path>`

---

## 6. Stockage et recherche

```
.ankor/
  config.yml
  allowed_signers   # cles publiques autorisees a ratifier (§8), versionne
  tasks/TASK-<id>.md
  adr/ADR-<id>.md
  index.db          # derive, jetable, gitignore
```

**Arborescence plate.** Le rattachement se fait par `scope`, pas par emplacement. Une entité peut contraindre plusieurs modules ; une arborescence miroir du code forcerait un parent unique et casserait au premier refactor.

**Écritures atomiques** (write-then-rename), sous un verrou de fichier le temps du cycle lecture-comparaison-écriture — c'est ce qui rend le compare-and-swap de `version` effectif, write-then-rename seul ne comparant rien.

**Index SQLite dérivé**, entièrement reconstructible depuis les fichiers. Il n'est jamais source de vérité.

**Cycle de vie de l'index, fixé** (ex-point ouvert 1) : l'index mémorise un hash de contenu par fichier `.ankor/`. Chaque commande compare les fichiers du périmètre touché à ces hashes et réindexe incrémentalement ce qui a divergé — l'index est donc toujours à jour *au moment de la lecture*, sans daemon ni watcher. `check` réindexe intégralement. Un index absent ou d'un schéma inconnu est reconstruit silencieusement : le supprimer est toujours une opération sûre.

### Trois niveaux de recherche

1. **Résolution par scope** — match de globs, déterministe, zéro ambiguïté. Couvre l'essentiel des cas, c'est ce que fait `context`.
2. **Recherche lexicale** (FTS5) pour `find`. Rapide, locale, explicable.
3. **Sémantique** — explicitement hors périmètre. Elle imposerait un modèle d'embeddings (perte d'agnosticité) et un ranking non déterministe, ce qui réintroduit exactement l'incertitude que l'outil cherche à éliminer.

---

## 7. Synchronisation

Modèle git-like : pleinement fonctionnel en local, déployable progressivement, jamais dépendant d'un service.

### Modèle d'exécution nominal

Le cas nominal est **un arbre de travail par agent** — clones ou `git worktree` — chacun sur sa branche. C'est ce que supposent déjà les preuves locales (un `verify` exécuté dans un arbre que d'autres agents modifient en parallèle ne prouverait rien de net) et c'est la pratique effective des harnais agentiques. Le partage d'un même arbre par plusieurs agents fonctionne mais est un mode dégradé, pas un mode de conception.

### Séparation des deux plans

- **État durable** (tâches, ADR, log) — répliqué, versionné, offline-first. Git pur.
- **Coordination éphémère** (claims) — TTL court, jamais historisé, jetable. Seule chose nécessitant un arbitre.

Les claims vivent dans des refs git dédiés, **un ref par tâche** : `refs/ankor/claims/<task-id>`. Un ref unique pour tous les claims mettrait en contention des claims sans rapport — deux agents prenant deux tâches différentes se disputeraient le même push non-fast-forward. Par tâche, le CAS de git arbitre exactement le conflit qui compte et aucun autre. Ces refs ne sont jamais mergés dans la branche de travail : aucun bruit dans les diffs, aucun conflit sur les fichiers.

Un enregistrement de claim porte : identité, horodatage d'expiration, hash du `done_criteria` gelé (§3), et hash de l'ensemble des contraintes applicables au scope de la tâche au moment du claim. Ce dernier ferme la fenêtre du travail long : une contrainte acceptée pendant que l'agent travaille change ce hash, et `done` **avertit** — jamais ne bloque, la contrainte nouvelle ne concerne pas nécessairement le travail déjà fait — en invitant à relire `ankor context` ; `check` remonte le même cas. Le ref est **supprimé** à `done`, `release` ou reprise après expiration, et `check` élague les refs orphelins — « jamais historisé » est une opération d'entretien, pas une propriété gratuite. `ankor init` ajoute le refspec de fetch `refs/ankor/*` à la config du repo : les hébergeurs ne rapatrient pas les refs non standard d'eux-mêmes.

L'expiration est évaluée sur l'horodatage porté par le claim, avec une tolérance de dérive d'horloge de 2 minutes : à l'échelle d'un TTL de 30 minutes, NTP suffit largement.

### Pourquoi `version` coexiste avec le CAS de git

Les deux couvrent des portées disjointes. Le CAS de git protège **entre clones**, au moment du push. Le champ `version` protège **à l'intérieur d'un même arbre de travail**. Avec un arbre par agent, ce cas devient rare — mais pas nul : un humain et un agent partagent souvent le même arbre, et le coût du champ est un entier. On le garde pour le cas résiduel, sans plus le présenter comme la défense principale.

### Niveau 0 — local

Pas de remote. Les claims utilisent les **mêmes refs `refs/ankor/claims/<id>`, en local** : une mise à jour de ref git locale est déjà atomique, et le niveau 1 devient littéralement « le même ref, poussé » — aucune migration, aucun état à convertir. Il n'existe **pas de repli sans git** : git est une dépendance dure, un repo non initialisé sort en code 9 avec la commande exacte. Fonctionnel sans configuration, comme un `git init` sans push. Mode par défaut.

### Niveau 1 — remote git seul

N'importe quel remote existant, GitHub inclus. Zéro infrastructure.

L'insight central : **une mise à jour de ref git est déjà un compare-and-swap atomique**. Un push non-fast-forward échoue côté serveur, atomiquement, chez tous les hébergeurs. C'est exactement la primitive nécessaire aux claims — le CAS est garanti par git, pas par du code maison.

Le renouvellement de TTL par `log` met à jour le ref local puis pousse ; à raison d'un log toutes les quelques minutes et d'un push de l'ordre de la seconde, le coût est marginal. **Hors ligne au niveau 1**, le claim est pris localement et marqué non synchronisé, avec avertissement : dégradation, pas échec — le risque de claim concurrent est affiché, pas masqué.

Contrepartie : latence de l'ordre de la seconde, pas de notification. Confortable jusqu'à deux ou trois agents, sature au-delà.

### Niveau 2 — `ankor serve`

Binaire unique, un port, un SQLite. Ne stocke **que les claims**, diffuse les changements en SSE. L'état durable continue de passer par git ; le daemon ne le possède jamais.

Conséquence : si le daemon tombe, repli automatique au niveau 1, sans perte possible.

Le passage d'un niveau à l'autre ne change ni le format, ni les commandes.

### Merge de l'état durable

Deux branches ayant modifié la même tâche se rencontrent comme n'importe quel conflit git : la v1 n'embarque **pas de driver de merge dédié**, la résolution est humaine, et `check` détecte les marqueurs de conflit résiduels dans `.ankor/` (code 8) pour qu'un merge bâclé ne passe pas la CI. Deux règles de résolution guident l'humain et préparent un driver futur : `version` résolue = max des deux + 1 ; section `## Log` = union ordonnée par horodatage (append-only, l'union est toujours correcte). Le driver de merge automatisant ces deux règles est un candidat v1.1 (§13).

---

## 8. Permissions

Modèle déclaratif dans `.ankor/config.yml`, identité via `$ANKOR_AGENT`.

```yaml
roles:
  agent:
    can: [context, find, claim, log, done, new:task, new:adr:proposed]
    cannot: [adr:accept, adr:edit-constraint, task:close, delete]
  human:
    can: ["*"]
identities:
  "marie@laptop": human      # tout identifiant absent de la table = role agent
```

**Le rôle par défaut est `agent`.** Une identité inconnue — y compris `$ANKOR_AGENT` absent, auquel cas l'identité de repli est `<user>@<hostname>` — reçoit le moindre privilège. Se déclarer humain dans la config ne confère de toute façon aucune autorité réelle : c'est la signature qui la porte.

Le garde-fou principal n'est pas la permission mais **le statut** : un agent écrit librement, en `proposed`. Il capture l'information immédiatement sans pouvoir contraindre qui que ce soit. La ratification humaine est le seul chemin vers l'autorité.

### Ancrage de l'identité humaine

`$ANKOR_AGENT` est posé par l'agent lui-même : un agent qui déraille peut se déclarer `human`. Aucune vérification au niveau du fichier ne peut l'en empêcher, puisqu'il a accès au système de fichiers.

Le seul ancrage qui tienne est donc externe : **la ratification exige un commit signé**. `accept` produit lui-même le commit de ratification (§12), qui enregistre le SHA et le hash de `constraint` + `scope` de l'ADR accepté, et `check` vérifie la signature via `git verify-commit` contre les clés de `.ankor/allowed_signers` (format SSH allowed-signers de git ; GPG supporté via la config git standard). Le fichier de clés est versionné : y ajouter une clé est un diff en revue.

**Ce que la signature prouve — et ne prouve pas.** Elle prouve l'accès à une clé autorisée, pas une intention humaine. Un agent qui tourne sur la machine d'un développeur dont la signature git est configurée et déverrouillée peut produire un commit signé valide. La défense contre ce cas est opérationnelle, pas cryptographique : clé de ratification protégée par passphrase ou matériel (touch-to-sign), distincte si besoin de la clé de commit courante. C'est cohérent avec le modèle de menace (§1) : on protège contre la dérive, pas contre un adversaire.

Sans signature configurée, les permissions sont **consultatives** — elles protègent contre la dérive accidentelle, pas contre un agent qui cherche activement à les contourner. C'est une limite assumée et affichée par `check`, pas masquée.

---

## 9. Amorçage

L'installation du skill s'appuie sur l'écosystème existant plutôt que sur un mécanisme maison :

```
npx skills add <owner>/ankor
```

Les skills s'installent depuis des dépôts et non depuis des paquets npm : l'identifiant est `owner/repo`, il n'existe pas de nom nu. Le dépôt du skill s'appelle donc simplement `ankor`. L'installation se fait par lien symbolique, et un `skills-lock.json` versionné dans le repo reproduit le même jeu de skills sur toutes les machines de l'équipe — cohérent avec le reste du design, où tout ce qui compte est dans le repo.

Le CLI `skills` gère la détection multi-agents (Claude Code, Codex, Cursor, OpenCode, et bien d'autres) et crée des liens depuis chaque agent vers une copie canonique — exactement le design souhaité, déjà maintenu par un tiers.

**Économie de tokens.** Ces fichiers sont chargés en permanence. Le SKILL.md porte donc les sept commandes et le modèle mental ; le détail des flags reste dans `ankor help`, chargé à la demande.

`ankor init` garde un périmètre étroit : créer `.ankor/`, écrire `config.yml`, ajouter le refspec `refs/ankor/*` (§7), poser un pointeur dans `AGENTS.md`.

**Distribution du binaire** : le skill dit *comment* utiliser Ankor, il n'installe pas le CLI. Prévoir `curl | sh` et Homebrew en plus de npm.

---

## 10. Périmètre v1

### Dans

Format de fichier · surface agent (7 verbes) et surface humaine (`review`, `accept`, `check`, `show`) · HEAD · `release` (raison obligatoire) · IDs et résolution par préfixe · scope déclaré obligatoire · `blocked_by` et blocage dérivé · vérificateurs nommés en liste, timeout, exécution locale des preuves · gel par hash (`done_criteria` au claim, `constraint`/`scope` à la ratification) · codes de sortie · `context` biphasé · niveaux de sync 0 et 1, claims sur refs par tâche · permissions déclaratives ancrées sur commit signé, `allowed_signers` versionné · skill d'amorçage · `check` (périmètre complet en §4).

### Hors v1, par ordre de valeur attendue

| Reporté | Raison |
|---|---|
| `--since` (contexte différentiel) | Fort gain token sur les loops longues, mais exige un état « vu » par agent. Premier candidat pour v1.1. |
| `ankor attest <id> --proof ci://<provider>/<run-id>` | Forme figée dès maintenant ; la structure `proof` en liste append-only est prête, la commande viendra quand une CI l'appellera. |
| Driver de merge `.ankor/` | Les règles de résolution sont fixées (§7) ; leur automatisation attendra les premiers conflits réels. |
| `touched` inféré depuis les commits | Détection de dérive de scope. Dépendance git, non bloquante pour démarrer. |
| `enforced_by` (mécanisation) | Le mécanisme de fond contre l'inflation de contexte (voir §11). Inutile tant que le corpus d'ADR est petit. |
| `ankor serve` (niveau 2) | Le niveau 1 suffit jusqu'à trois agents concurrents. |
| `ankor review --coherence` (analyse du corpus d'ADR) | Détection de contradictions et de doublons. Sans valeur sur un petit corpus. La file de ratification, elle, est en v1. |
| Vue web read-only | À rouvrir seulement si des non-développeurs doivent lire le board. |
| Export Linear/Jira | Visibilité management. Jamais en écriture vers Ankor. |
| Types d'entités additionnels | Le socle commun rend l'extension triviale plus tard. Ne pas anticiper. |

---

## 11. Cycle de vie des contraintes

Non implémenté en v1, mais le modèle doit être posé maintenant car il conditionne le champ `enforced_by`.

Le problème : si les ADR s'accumulent sans jamais mourir, le contexte grossit indéfiniment et l'outil devient le problème qu'il devait résoudre. Un plafond chiffré ne fait que déplacer l'arbitraire.

**La mécanisation est le puits naturel.** Une contrainte naît en prose parce qu'on ne sait pas encore la vérifier. Beaucoup deviennent mécanisables — « pas de `jwt.verify` dans `src/auth` » est une règle de lint. Une fois en CI, elle n'a plus rien à faire dans le contexte : la boucle de feedback l'attrape mieux, et de façon déterministe. Le champ `enforced_by` la sort du contexte injecté sans la désactiver.

Cela retourne la pression dans le bon sens : un contexte qui grossit incite à écrire des checks, pas à supprimer des décisions.

**Trois signaux complémentaires, tous non arbitraires :**

- **Pression relative** — quelle fraction du budget de `context` les contraintes consomment sur un scope. S'auto-échelle, ne dépend d'aucun nombre choisi à la main.
- **Rétrécissement de scope** — une contrainte déclarée sur `src/**` dont les tâches liées n'ont jamais touché que `src/auth/**` est sur-déclarée. Précision gagnée, information conservée.
- **Mort structurelle** — scope qui ne matche plus aucun fichier, chaîne de supersede rompue. Vérifiable, contrairement à une décroissance temporelle : une contrainte de trois ans peut être vitale. La même règle vaut pour les tâches : une tâche dont le scope est mort est signalée par `check`, jamais fermée automatiquement — le code a peut-être simplement été déplacé.

**Règle absolue : aucune suppression automatique.** Une contrainte jamais violée ressemble exactement à une contrainte inutile. L'outil détecte et propose ; un humain ratifie. Le seul automatisme autorisé est le retrait du contexte injecté pour ce qui est mécanisé — et là c'est sûr, la CI a pris le relais.

---

## 12. Implémentation

**Rust.** Justifié ici pour trois raisons alignées avec les objectifs : binaire statique sans runtime (l'agnosticité d'agent suppose de ne rien imposer à l'environnement hôte), écosystème adapté (`rusqlite`, `globset`), et un typage qui rend les invariants de la machine à états — transitions illégales, champs gelés — vérifiables à la compilation plutôt qu'à l'exécution.

**L'analyse d'arguments est faite à la main**, sans bibliothèque. La raison n'est pas l'économie d'une dépendance mais le contrôle au caractère près de deux surfaces lues par des agents : les erreurs auto-correctives (§4), qu'un analyseur générique remplacerait par ses propres messages, et `ankor help`, dont §9 dit qu'il porte le détail des flags — une aide générée est verbeuse, et son coût est payé à chaque appel qui la déclenche. La surface étant figée à douze commandes (§4), le coût de l'écrire à la main ne croît pas.

Le coût réel est la vitesse d'itération sur un design encore mouvant. Mitigation : figer et implémenter le **parseur de format** en premier, indépendamment du CLI. C'est la partie stable et la seule dont dépend l'interopérabilité.

### Ankor et git : qui commite

**Ankor ne commite jamais, à une exception près.** Les écritures de `new`, `log`, `done`, `release` atterrissent dans l'arbre de travail et se propagent au rythme des commits de l'agent, avec son code — l'état organisationnel et le code qu'il décrit voyagent ensemble, ce qui est exactement la promesse de l'outil. Conséquence assumée : au niveau 1, un autre clone ne voit une transition qu'une fois le commit poussé ; la coordination temps réel, elle, passe par les claims, qui ne dépendent d'aucun commit.

L'exception est **`accept`, qui produit lui-même le commit signé de ratification**, ne contenant que le fichier de l'ADR promu (et, le cas échéant, l'ADR remplacé passant en `superseded`). Le modèle d'autorité repose sur ce commit ; le laisser à la discrétion de l'appelant le rendrait optionnel.

### Récupération

Ankor n'implémente aucun mécanisme d'annulation, de corbeille ou d'historique propre, et c'est délibéré : **git est déjà tout cela**. Un `done` erroné, un ADR remplacé par erreur, un fichier supprimé se récupèrent avec les outils que l'utilisateur connaît déjà.

La conséquence à respecter dans l'implémentation : chaque opération doit produire un diff propre et lisible. Le format du log — section append-only en fin de fichier (§3) — est choisi précisément pour cela : chaque `log` est un diff d'une ligne, chaque transition un diff de frontmatter minimal.

---

## 13. Décisions finales

**Licence : GPL-3.0.** Le critère retenu : modification et commercialisation libres, mais un fork distribué doit publier ses sources — c'est la définition du copyleft fort. Deux précisions honnêtes sur ce que la GPL garantit réellement : l'obligation de publier ne se déclenche qu'à la *distribution* (un fork gardé interne n'y est pas tenu, aucune licence classique ne l'impose), et elle porte sur le code du CLI, pas sur le format — les fichiers `.ankor/` des utilisateurs et les outils tiers qui les lisent ou les écrivent ne sont pas des œuvres dérivées, ce qui préserve « le format est la spec ». Un service hébergé bâti sur Ankor n'est pas contraint (la clause réseau serait l'AGPL) ; pour un CLI local, ce cas est marginal et l'AGPL freinerait l'adoption pour rien.

**Plateformes : Linux, macOS, Windows, natifs en v1.** Rust cross-compile les trois sans friction, `gix` évite la dépendance au binaire git système pour la plomberie, et les vérificateurs s'exécutent partout via le `sh` de Git for Windows (§4). Distribution : `curl | sh`, Homebrew, Scoop/winget, npm.

**Différés en v1.1, forme figée dès maintenant** : le driver de merge `.ankor/` (règles fixées en §7 — `version` = max + 1, log = union horodatée — automatisées aux premiers conflits réels) et `ankor attest` (forme de commande figée en §10, implémentée quand une CI l'appellera).

Plus aucun point ouvert : le format, la boucle agent, les trois plateformes et les niveaux 0 et 1 sont entièrement spécifiés.
