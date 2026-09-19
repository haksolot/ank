---
id: LOG-b7be59c49d3a
type: log
title: "Measured 2026-09-13 on Windows, every command the diff writes, each isolated: HOME, USERPROFILE,"
created: 2026-09-13T13:30:12Z
author: claude-code/b2c8
scope:
  - README.md
  - docs/getting-started.md
  - docs/agents.md
  - npm/ank/README.md
about: TASK-b2c8b4bc58df
seq: 2
schema: 4
version: 1
---

 APPDATA, LOCALAPPDATA and npm_config_cache pointed at a scratch home, cwd a scratch dir, CLAUDE_CONFIG_DIR scratch for the plugin. Real ~/.claude/skills, ~/.agents, ~/.codex/skills, ~/.cursor/skills, ~/.pi, ~/.claude/plugins, ~/.claude.json and ~/.claude/settings.json listed with mtimes before and after: 148 and 80 entries, diff empty. Results: ank skills (a1f599a) printed 6 lines, exit 0. ank skills --install wrote 6 skills to %TEMP%\ank-skills-25808-138072400-0, ran npx skills add <dir>, which detected claude-code and installed 6 skills as copies (test -L false) into ./.agents/skills of the cwd plus skills-lock.json, exit 0; after rm -rf of the temp dir the 6 SKILL.md remain and ank/SKILL.md is cmp-identical to skill/SKILL.md. npx fetched the skills CLI into the empty cache (7.9M). With PATH holding only the binary and /usr/bin, --install printed 'npx is not on PATH, so nothing was installed' and the command, exit 0. npx skills add haksolot/ank --list: cloned, Found 6 skills, exit 0; without --list: installed 6, exit 0. pi install npm:@haksolot/ank exit 0; pi install git:github.com/haksolot/ank exit 128 first (Filename too long: an artifact of the long scratch path), exit 0 with core.longpaths=true. claude plugin marketplace add haksolot/ank exit 0, claude plugin install ank@ank exit 0 (scope user, in the scratch config dir). npm install -g @haksolot/ank --prefix scratch exit 0, gives 0.7.0 without the verb (see LOG-3940a82ee184).
