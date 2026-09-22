# The specification

The normative text behind every page of this site lives in the corpus, as
`spec` entities, and not here. A page explains and links; a spec states the rule,
and where the two disagree the spec is right and the page is a bug
(ADR-33970fcdb6e8). They argue the design; they are not a tutorial.

Each one says in its own body which sections of the original single document it
carries, so a rule that reads `(§7)` is resolved by the spec that claims §7. In a
checkout, `ank find --type spec` lists them and `ank show <id>` prints one whole.
On the web, each link below opens the file as the default branch has it.

- [SPEC-1d5b44efd388](https://github.com/haksolot/ank/blob/main/.ank/entities/SPEC-1d5b44efd388.md) Intent, principles, and what v1 leaves out
- [SPEC-ac4aad6d1edb](https://github.com/haksolot/ank/blob/main/.ank/entities/SPEC-ac4aad6d1edb.md) The data model
- [SPEC-cf285efcdca4](https://github.com/haksolot/ank/blob/main/.ank/entities/SPEC-cf285efcdca4.md) Storage and search
- [SPEC-15a56aeedcfd](https://github.com/haksolot/ank/blob/main/.ank/entities/SPEC-15a56aeedcfd.md) Synchronisation
- [SPEC-77d99d8d1ef2](https://github.com/haksolot/ank/blob/main/.ank/entities/SPEC-77d99d8d1ef2.md) Proof, anchoring and authority
- [SPEC-219033e25653](https://github.com/haksolot/ank/blob/main/.ank/entities/SPEC-219033e25653.md) The CLI surface
- [SPEC-89070ce7f3b8](https://github.com/haksolot/ank/blob/main/.ank/entities/SPEC-89070ce7f3b8.md) Presentation: structure for every reader, colour for a terminal
- [SPEC-a1234da5449a](https://github.com/haksolot/ank/blob/main/.ank/entities/SPEC-a1234da5449a.md) The attention budget and the constraint lifecycle
- [SPEC-3bccb8aee5b7](https://github.com/haksolot/ank/blob/main/.ank/entities/SPEC-3bccb8aee5b7.md) Bootstrapping, teaching and distribution
- [SPEC-93531977642f](https://github.com/haksolot/ank/blob/main/.ank/entities/SPEC-93531977642f.md) Implementation, and the decisions that bound it

The list is every spec the corpus holds as accepted, and a test holds it to
`ank find --type spec` on this repository: a spec accepted, superseded or
retitled without this page following turns the suite red. A superseded spec is
left off, and `ank show` still prints it, with the one that replaced it named.

**Linked, not rendered.** The specs are read here through GitHub's view of the
file rather than copied into the site at build time. A rendered copy would need
the site's build to run `ank`, and would be one more place the text could be
read out of step with the corpus it came from; the link always reaches the
default branch's bytes.
