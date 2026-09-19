---
id: LOG-caa1ebf85848
type: log
title: WHAT I TOUCHED OUTSIDE crates/ank-tui/**, said explicitly because the scope is the reader's.
created: 2026-08-27T05:49:01Z
author: claude-code/opus-5+config
scope:
  - crates/ank-tui/**
about: TASK-b08d090f699c
seq: 3
schema: 4
version: 1
---

 crates/ank-contract/src/verbs.rs gains CONFIG_KEYS and CONFIG_KEYS_NOTE -- the eight keys written once, in a macro invocation that expands to both the slice and the space-separated sentence config's own note carries -- and the note is now that constant. crates/ank-cli/src/config.rs's KEYS is that constant rather than a second literal, and the doc on cli.rs's the_config_note_names_every_key_the_verb_addresses says what it measures now: a rendering and the shape a reader with only the note to go on would split it by, rather than two memories being arbitrated. That is the body's recorded intent, one table instead of three, and it is the only reason the reader could reach the set at all -- ADR-8bd76e8d7c4e forbids it linking ank-cli, and a space-separated line inside a note is a string to split rather than a list to read.

CONCAT AND NOT A JOIN, because the note is a &'static str inside a const table: joining a slice at run time needs an allocation the table cannot hold, and concat! takes literals rather than constants. The macro is what lets the literals be written once and expanded into both shapes.

THE GATE IS THE ONE THE BODY RECORDED AND I DID NOT INVENT ANOTHER. ank::BOTH_ROADS names exactly one verb and buys nothing on its own; ank::reading_shape is what decides, and Ank::json admits a verb named there only in that shape. One positional is a read; everything else is refused before anything is spawned. I made it stricter than 'two positionals is a write' in one direction and the reason is on the function: --unset is ONE positional and removes a line from the file, so a gate that counted positionals alone would have left a write on the reading road -- which is the exact hole the exception exists to not open. 'Exactly one word and it is not a flag' covers the value, --unset and --user together, and errs towards refusing.

Failed gains NotAReadingShape rather than reusing NotARead, because the two say different things and only one is true: config IS a verb this reader reads with, and 'config <key> <value>' is not a read.

WHAT o OPENS IS A PANE AND NOT A FIFTH PANEL. LOG-1afc1b09f95b spends a whole entry on rows being a budget and a panel costing two of them at every window whether or not anybody looks at it. Pane::Config sits beside Pane::Body and Pane::Constraints, s and o toggle into their own through one App::pane_to, and the pane costs nothing at rest. It is the one pane that names no entity -- what a corpus is configured to do is a fact about the repository -- so pane_content answers it before the document is asked for, body_lines does not draw 'nothing is open here' over it, and title_of says CONFIG rather than BODY.

AND OPENING AN ENTITY LEAVES IT. show() is called by open_selected (a person asking) and by reopen (a refresh after a write), so the pane is dropped in the first and not the second: a person who pressed Enter on a listing row and got a list of settings would have been answered with something else, and a config write that flipped the pane back to the document would undo what they were doing. The constraints pane stays, because it follows the entity it is about.

THE FORM IS THE ONE THAT WAS ALREADY THERE, WIDENED BY ONE FACT. Form::on(verb, front) takes the positionals the caller has already decided, and form::taken reads the placeholder of the NEXT one off the verb's own positional_help -- '<key> [<value>]' word by word, brackets trimmed -- so <value> is a row of the form and composes as a bare word. taken answers None while nothing has been supplied, which is what keeps new/edit/close/attest untouched: their first positional is always somebody else's, the kind or the view's <id>. NEEDS gains (config, '', Need::Any([VALUE, '--unset'])) -- the reading shape is what it refuses, and either thing that writes is enough.

Need::Any gained a second field and it had to. Its legend said 'a call naming none opens $EDITOR', which is true of edit and false of config: ank config <key> READS. A screen telling a person that setting nothing opens an editor is worse than one that says nothing, so the sentence is a field of the variant now and each row carries its own.

--user is drawn on that form like every other flag the verb declares, deliberately: a form that hid one would be this reader deciding which half of a verb a person may reach, and what stands between the switch and the file is what stands in front of every act here -- the command line on the screen and one key.

THE TWO TRAPS THE TASK RECORDED WERE NOT REDISCOVERED, and the first one bit exactly as described. I mutated model::Settings::load to take only three of the eight keys and ran cargo test -p ank-tui --test config: it PASSED, against target/debug/ank as it was before the edit. cargo build --workspace and the same command then failed on the equality. Every pty result in this task was taken after a workspace build. The second trap cost nothing: ank.rs names no binding, no BINDINGS and no bindings:: anywhere, comments included, and the scan is green.

WHERE THE PRICE IS CHARGED. App::reconfig refuses unless the focus is the body panel AND the pane is Config, and it is called from three places: the o command, focus_on arriving at the body panel, and confirmed after a config write. reload does not call it and repaint reaches reload, so a watcher's news puts no ank config on the wire -- which matters more here than for the queue, because §4 gives config one key at a time and the pane is one spawn per declared key.

MEASURED, NOT ASSERTED. crates/ank-tui/tests/config.rs drives the binary on a pseudo-terminal at 140x40. The keys drawn are compared with the keys 'ank help config' declares, in order, so the pane and the CLI's own page have to agree; the value and the source on each row are compared with what 'ank config <key> --json' answers; peers.<name> is refused by the verb and is still a row carrying what it said, and both an answered key and a refused one are required to have been seen or the loop asserted nothing. Dismissing is measured on Repo::corpus(), every byte under .ank/ before and after. Confirming is measured by asking the binary again. Unsetting is measured on the SOURCE -- claim_ttl_default starts at 'default', goes to 'file', comes back to 'default' with the same value -- which is why that key and not one the file already carries: a key already in the file returns to 'file' either way and the assertion would say nothing. 'On no repaint' is measured by moving the corpus twice at the shell, a config key and a new task: the task arriving on the listing is what says the reload ran, and the config row not moving beside it is the claim.

I also checked the criterion's other half by hand rather than only by test: adding a ninth key to CONFIG_KEYS and rebuilding put nine rows on the pane and nine keys on 'ank help config', with no edit in crates/ank-tui at all. Reverted.
