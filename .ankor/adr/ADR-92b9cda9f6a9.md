---
id: ADR-92b9cda9f6a9
type: adr
slug: git-dependance-dure
title: git est une dépendance dure, il n'existe pas de mode sans git
created: 2026-07-28T00:09:51Z
status: proposed
scope:
  - crates/ankor-cli/**
constraint: |
  Un seul mécanisme de claim : les refs git refs/ankor/claims/<id>. Aucun
  repli par verrous de fichiers. La plomberie passe par le binaire git et
  jamais par une bibliothèque, et n'utilise que la plomberie (update-ref,
  rev-parse, verify-commit, hash-object, cat-file), jamais la porcelaine.
  Version minimale git 2.34, vérifiée au démarrage. git introuvable, trop
  ancien, ou répertoire de travail hors d'un repo git sortent en code 9
  avec la commande exacte à exécuter.
schema: 1
version: 2
---

Un repli par verrous de fichiers ne sauverait que le claim — la seule pièce
qui ne sert à rien seule. Sans git, Ankor perd la ratification (le commit
signé, §8), les preuves de type `commit`, la récupération qui tient lieu de
corbeille et d'undo (§12), et le refspec de synchronisation (§7). On
maintiendrait donc deux mécanismes de coordination pour un mode dégradé dans
lequel personne ne peut travailler.

Le principe « dégradation, pas échec » (§2) n'est pas affaibli, il est
précisé : la dégradation porte sur les services et le réseau — pas de remote,
pas de daemon — jamais sur le substrat. Un niveau 0 sans remote reste
pleinement fonctionnel, parce qu'une mise à jour de ref git locale est déjà
la primitive atomique dont le claim a besoin.

Le code 9 est le bon code et non le 1 : un environnement dépourvu de git
n'est pas un échec de la tâche de l'agent, c'est un environnement à réparer.

Le binaire plutôt qu'une bibliothèque (`gix`) ne découle pas de cette
décision, il a sa propre raison, plus forte : `accept` et `check` reposent
sur la signature. Produire un commit signé et le vérifier contre
`allowed_signers` est trois lignes avec `git commit -S` et
`git verify-commit`, et un chantier cryptographique avec une bibliothèque —
pour un résultat au mieux équivalent, au pire subtilement différent de ce
que l'utilisateur vérifiera à la main.

La restriction à la plomberie est ce qui rend le choix soutenable : la
porcelaine n'a aucun contrat de stabilité entre versions, et la parser
recréerait exactement la dette que le recours au binaire évite. Le plancher
2.34 est la version qui introduit la signature SSH et
`gpg.ssh.allowedSignersFile` : en dessous, la ratification ne peut pas
fonctionner, et le découvrir au premier `accept` serait tard.
