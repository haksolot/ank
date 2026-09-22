<!-- Generated from crates/ank-core/src/config.rs; do not edit.
     Regenerate: cargo run -q -p ank-core --bin config-keys > docs/config-keys.md -->

# config.yml keys

Every key `.ank/config.yml` may carry, with the type of its value and what an absent key means. A key not listed here is refused, and so is a file whose `schema` is newer than this build reads.

The file declares `schema: 1`, the only version this build reads. A duration is `<n><unit>`, the unit one of `s`, `m`, `h` or `d`. `<name>` stands for a key the file chooses; `ank config <key>` reads and writes the scalar keys, and `roles` and `identities` are edited by hand.

| Key | Type | Default | Notes |
|---|---|---|---|
| `schema` | integer | required | the version of this file's format |
| `context_budget` | integer | `8000` | what `context` hands a reader, in characters |
| `claim_ttl_max` | duration | `2h` | the longest lease a claim is granted, whatever `--ttl` asks |
| `claim_ttl_default` | duration | `30m` | the lease `claim` grants without `--ttl`, capped by `claim_ttl_max` |
| `default_branch` | string | none | the branch carrying the reference state; absent, `refs/remotes/origin/HEAD` names it |
| `peers.<name>` | path | none | a peer corpus a scope entry reaches by name, relative to this root or absolute |
| `verifiers.<name>.run` | command | required | what `done` runs through `sh`; required in a declared verifier |
| `verifiers.<name>.timeout` | duration | `10m` | how long `done` lets the command run |
| `verifiers.<name>.default` | boolean | `false` | `true` writes the verifier into every task `ank new task` creates |
| `roles.<name>.can` | list of strings | `[]` | what the role may do, declared |
| `roles.<name>.cannot` | list of strings | `[]` | what the role may not do, declared |
| `identities.<identity>` | string | none | the role of an identity; one absent from the table is an `agent` |
| `weight.hot_files` | integer | `3000` | `check` signals a hot corpus holding more entity files |
| `weight.plane_bytes` | integer | `4000000` | `check` signals claim and proof records weighing more bytes |
