---
id: LOG-10deddc4f468
type: log
title: Ran the route before writing it, with the binary built from 98999e3 (the PATH copy was 8df04cb,
created: 2026-09-15T10:27:37Z
author: claude-code/opus-5+guide-level-one
scope:
  - docs/getting-started.md
about: TASK-7eccd56c8c30
seq: 1
schema: 4
version: 1
---

 stale), in a scratch lab under TMPDIR with a throwaway git identity and gpgsign off. A bare origin.git, one task seeded and pushed, then cloned twice over file://. Clone one: ank claim exit 0. Clone two: exit 4, 'error[4]: TASK held by agent-one (expires in 30m)'. refs/ank/claims/<id> on the bare origin and in both clones: one blob, same sha. Neither clone had +refs/ank/* in its fetch refspec (git clone does not copy it), and arbitration held anyway: claim reads origin itself. Released from clone one (the deletion reached origin: for-each-ref refs/ank on it came back empty), then cloned twice over ssh://lan-host/<path> with GIT_SSH_COMMAND set to a shim that runs the remote command locally, so the ssh transport was exercised and the network leg was not: clone five exit 0, clone six exit 4 with the holder named, one claim ref on origin; the shim logged 6 invocations (git-upload-pack and git-receive-pack). No common origin: two clones with origin removed, stale claim ref deleted in each; both ank claim exit 0, two different claim blobs (e15c75c vs 0daf1dd), status in each says 'elsewhere no claim by another agent', status --remote warns only that there is no origin, check exits 0 with no finding about it. Binary agrees with SPEC-15a56aeedcfd on all three statements. Level is decided on remote.origin.url (git.rs:160), so a URL-less origin is level 0 and silent: a remoteless repo's claim after init printed nothing extra. One defect found, not in scope and not fixed here: ank init in a repository with no remote writes [remote "origin"] fetch=+refs/ank/*:refs/ank/* with no url, after which 'git remote add origin <url>' fails with 'error: remote origin already exists' (exit 3), and that is the exact command status --remote's warning names. git remote set-url origin <url> works but leaves no +refs/heads/* fetch refspec. The guide paragraph does not name either command for that reason.
