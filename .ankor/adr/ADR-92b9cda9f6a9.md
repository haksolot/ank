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
  repli par verrous de fichiers. git introuvable, ou répertoire de travail
  hors d'un repo git, sort en code 9 avec la commande exacte à exécuter
  (git init, ou le lien d'installation de git).
schema: 1
version: 1
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
