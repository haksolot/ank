---
id: LOG-7bbadf866acb
type: log
title: Measured at d541d13b (target/debug/ank; the PATH copy is a1f1d59d, stale) in a lab corpus under
created: 2026-11-19T03:11:44Z
author: claude-code/opus-5.5+w9-spec-successors
scope:
  - .ank/entities/SPEC-*.md
about: TASK-876a641d3939
seq: 1
schema: 4
version: 1
---

 $TMPDIR/lab with a bare origin: reject without --reason exit 1, under --json stdout 0 bytes and one stderr line {contract:1,error:{code:1,name:Generic,message,next:[ank reject <id> --reason ...],subject:null}}; accept of a successor whose target is proposed exit 7 naming ank reject <target> --reason; reject exit 0 writes an annotated tag at refs/ank/reject/<id>, pushed to origin, pointing at the blob of the file as written (hash-object equal), message 'reject <id>' + by: + reason:, signed where git signs; entry 'rejected: <reason>'; no ratified/anchor written into the file; reject again exit 7; reject of an accepted ADR exit 7 naming ank new adr --supersedes; accept of the successor of a rejected exit 0, target stays rejected; review drops it, find --status rejected lists it; status: rejected typed by hand -> check fault 'reads rejected and no refs/ank/reject/<id> anchors it' exit 8; archive --dry-run with two rejected documents: 'nothing is cold' (the cold rule moves superseded only). hooks status: 4 text lines + corpus, --json {contract:1,agents[{agent,settings,installed,events,binary,current}],corpus,guards[{key,declared,in_force,variable,set,ignored}]}, an argument exit 1, outside a corpus 'no corpus is in force here, so no guard applies'. generic: context/deny/allow+message one line, mismatched event and garbage input silent exit 0, run --json exit 1 as a document. MCP: tools/list 33 tools; a refused tools/call returns isError, exitCode 2, structuredContent = the stderr document. help --json: 33 verbs.
