# ankor

Rend les taches et contraintes du repo lisibles en un appel. Sept verbes :

    Boucle :      ankor context -> ankor claim <id> -> ankor log "<msg>" -> ankor done
    Hors-boucle : ankor new, ankor find, ankor release --reason "<r>"

- `context` avant tout : oriente (taches + contraintes du perimetre), puis
  pilote l'execution une fois un claim pose (criteres, contraintes, log).
- `done` execute les verificateurs lui-meme ; ne jamais s'auto-rapporter.
- Bloque ? `release --reason` plutot que laisser expirer.

(Embryon — le CLI est en construction ; ce fichier deviendra le SKILL.md
installe via `npx skills add`.)
