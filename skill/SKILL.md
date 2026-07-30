# ank

Makes a repo's tasks and constraints readable in one call. Seven verbs:

    Loop:      ank context -> ank claim <id> -> ank log "<msg>" -> ank done
    Off-loop:  ank new, ank find, ank release --reason "<r>"

- `context` first: it orients you (tasks + constraints for the perimeter), then
  drives execution once a claim is held (criteria, constraints, log).
- `done` runs the verifiers itself; never self-report.
- Stuck? `release --reason` rather than letting the claim expire.

(Embryo — the CLI is under construction; this file will become the SKILL.md
installed through `npx skills add`.)
