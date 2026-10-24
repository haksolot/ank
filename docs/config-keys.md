<!-- Generated from crates/ank-core/src/config.rs; do not edit.
     Regenerate: cargo run -q -p ank-core --bin config-keys > docs/config-keys.md -->

# config.yml keys

Every key `.ank/config.yml` may carry, with the type of its value and what an absent key means. A key not listed here is refused, and so is a file whose `schema` is newer than this build reads.

This build reads `schema: 1` through `schema: 2`; a file carrying `requires_ank` declares `schema: 2`, and one without it may stay at `schema: 1`. A duration is `<n><unit>`, the unit one of `s`, `m`, `h` or `d`. `<name>` stands for a key the file chooses; `ank config <key>` reads and writes the scalar keys, and `roles` and `identities` are edited by hand.

| Key | Type | Default | Notes |
|---|---|---|---|
| `schema` | integer | required | the version of this file's format |
| `requires_ank` | version | none | `MAJOR.MINOR.PATCH`: the oldest ank this corpus needs, compared with the running binary and never with a release; a file carrying it declares `schema: 2`; absent, no requirement; moved by `ank config requires_ank` |
| `context_budget` | integer | `8000` | what `context` hands a reader, in characters |
| `claim_ttl_max` | duration | `2h` | the longest lease a claim is granted, whatever `--ttl` asks |
| `claim_ttl_default` | duration | `30m` | the lease `claim` grants without `--ttl`, capped by `claim_ttl_max` |
| `default_branch` | string | none | the branch carrying the reference state; absent, `refs/remotes/origin/HEAD` names it |
| `peers.<name>` | path | none | a peer corpus a scope entry reaches by name: a path to a checkout on disk, relative to this root or absolute, and never a URL; a reader whose layout differs overrides it in their own corpora.yml with `ank config --user peers.<identity>.<name> <path>` |
| `verifiers.<name>.run` | command | required | what `done` runs through `sh`; required in a declared verifier |
| `verifiers.<name>.timeout` | duration | `10m` | how long `done` lets the command run |
| `verifiers.<name>.default` | boolean | `false` | `true` writes the verifier into every task `ank new task` creates |
| `verifiers.<name>.inputs` | list of globs | none | the files whose content can change the result, relative to the root; absent, the whole tree; written as one flow sequence, never empty |
| `proof.reuse_max_age` | duration | `24h` | how old a passing run recorded under `<git-common-dir>/ank/runs/` may be for `done` to reuse it rather than run the verifier again; `done --fresh` reuses nothing |
| `verifier_jobs` | integer | none | how many verifiers one `done` runs at once, at least 1; absent, the machine's available parallelism |
| `roles.<name>.can` | list of strings | `[]` | what the role may do, declared |
| `roles.<name>.cannot` | list of strings | `[]` | what the role may not do, declared |
| `identities.<identity>` | string | none | the role of an identity; one absent from the table is an `agent` |
| `weight.hot_files` | integer | `3000` | `check` signals a hot corpus holding more entity files |
| `weight.plane_bytes` | integer | `4000000` | `check` signals claim and proof records weighing more bytes |
| `signers.<principal>.keys[].key` | string | required | a key allowed to ratify, `gpg <fingerprint>` or `<ssh key type> <base64>`; the block is written by `ank signers`, never by `ank config` (ADR-5c6db7a161b8) |
| `signers.<principal>.keys[].note` | string | none | what the key is, for the reader; `ank signers migrate` puts the comments of a former `.ank/allowed_signers` here |
| `signers.<principal>.keys[].valid_before` | date | none | `YYYY-MM-DD`: the key ratifies nothing committed from that day on, UTC, and what it signed before stays verified; absent, the key has no end |
