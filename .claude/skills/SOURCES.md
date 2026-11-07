# Agent skills

Skills for Claude Code (and other agents that read `.claude/skills/<name>/SKILL.md`), copied from
their authors' repositories at the commits below. Each folder keeps its upstream license.

## Installed by `scripts/install-skills.sh`

Two sources are installed on setup rather than committed, so this public repo doesn't republish them.
Run `scripts/install-skills.sh` once after cloning; it fetches the pinned commits into these git-ignored folders.

| Source | Scope | Skills | Why not committed |
| --- | --- | --- | --- |
| [Jakubantalik/transitions.dev](https://github.com/Jakubantalik/transitions.dev) | Full pack — transitions | `transitions-dev`, `transitions-polish`, `transitions-ui-generation` | Its license allows installing and using these, not redistributing the collection, which `transitions-dev` embeds. Its MIT tooling skills (`transitions-agent`, `refine-live`) are committed below. |
| [jnsahaj/skills](https://github.com/jnsahaj/skills) | Full pack — delivery and code quality | `code-refactor-review`, `create-draft-pr`, `explain-then-fix`, `ga`, `ga-pr`, `pit-of-success`, `prepare-branch-context`, `ux-flow-plan`, `zero-tech-debt` | No license file, so redistribution isn't granted. |

## Committed

| Source | Commit | License | Scope | Skills |
| --- | --- | --- | --- | --- |
| [mattpocock/skills](https://github.com/mattpocock/skills) | `4ae2317ddb22` | MIT | Full pack — engineering, planning, architecture | `ask-matt`, `code-review`, `codebase-design`, `diagnosing-bugs`, `domain-modeling`, `grill-with-docs`, `implement`, `implement-spec`, `improve-codebase-architecture`, `pr`, `matt-prototype`, `research`, `retro`, `setup-matt-pocock-skills`, `tdd`, `to-spec`, `to-tickets`, `triage`, `wayfinder`, `wizard`, `chief-of-staff`, `claude-handoff`, `loop-me`, `setup-ts-deep-modules`, `software-factory`, `writing-beats`, `writing-fragments`, `writing-shape`, `git-guardrails-claude-code`, `migrate-to-shoehorn`, `scaffold-exercises`, `setup-pre-commit`, `grill-me`, `grilling`, `handoff`, `teach`, `to-questionnaire`, `wait-what`, `writing-for-agents` |
| [emilkowalski/skill](https://github.com/emilkowalski/skill) | `c8a79627b59d` | MIT | Full pack — design engineering and animation | `animate`, `animate-expo`, `animation-performance`, `animation-vocabulary`, `apple-design`, `ask-sonner`, `break-ui`, `choreograph`, `emil-design-eng`, `find-animation-opportunities`, `improve-animations`, `mobile-native`, `pick-ui-library`, `emil-prototype`, `remove-animations`, `review-animations`, `write-swift` |
| [jakubkrehel/skills](https://github.com/jakubkrehel/skills) | `60bf140e43c9` | MIT | Full pack — interface quality and prototyping | `better-accessibility`, `better-colors`, `better-interface`, `better-layout`, `better-typography`, `better-ui`, `better-writing`, `break`, `build-design`, `explain-interface`, `interface-review`, `state-machine`, `teardown`, `variant` |
| [Jakubantalik/transitions.dev](https://github.com/Jakubantalik/transitions.dev) | `e3aefb554d8f` | MIT (tooling part of the Transitions.dev License) | Full pack — transitions | `transitions-agent`, `refine-live` |
| [humanlayer/skills](https://github.com/humanlayer/skills) | `4d69d20f6e9b` | MIT | Full pack — agent workflows | `build-iterated-agentic-loop`, `design-control-loop`, `improve-claude-md`, `narrow-react-prop-types`, `rpi`, `show-me`, `visual-pr` |
| [ast-grep/agent-skill](https://github.com/ast-grep/agent-skill) | `0613dc7f82bb` | MIT (per README) | Full pack — structural code exploration | `ast-grep`, `ast-grep-outline` |
| [shadcn/improve](https://github.com/shadcn/improve) | `cac56e1ebd3c` | MIT | Full pack — codebase improvement | `improve` |
| [vercel/turborepo](https://github.com/vercel/turborepo) | `efd19aa91692` | MIT | `turborepo` | `turborepo` |
| [cursor/plugins](https://github.com/cursor/plugins) | `2776ddd2355d` | MIT | Only `thermo-nuclear-review`, `thermo-nuclear-code-quality-review`, `thermos` | `thermo-nuclear-code-quality-review`, `thermo-nuclear-review`, `thermos` |
| [planetscale/database-skills](https://github.com/planetscale/database-skills) | `f6ed002d0308` | MIT | Only `postgres` | `postgres` |
| [vercel-labs/agent-browser](https://github.com/vercel-labs/agent-browser) | `71850a021c5c` | Apache-2.0 | Only `agent-browser` | `agent-browser` |
| [vercel-labs/agent-skills](https://github.com/vercel-labs/agent-skills) | `1a2c79d4e6c4` | MIT (per README) | Only `web-design-guidelines` | `web-design-guidelines` |
| [Astro-Han/karpathy-llm-wiki](https://github.com/Astro-Han/karpathy-llm-wiki) | `eafcc77001e4` | MIT | Knowledge-base skill | `karpathy-llm-wiki` |

cursor/plugins' `thermos` launches two subagents, which are in `.claude/agents/`
(`thermo-nuclear-review-subagent`, `thermo-nuclear-code-quality-review-subagent`, MIT, `LICENSE-thermos`).

## Instructions

> Preserve both authors’ `prototype` skills under the names `matt-prototype` and `emil-prototype` to avoid overwriting one another. Respect the selected-skill subsets above.

`matt-prototype` is mattpocock/skills' `prototype` and `emil-prototype` is emilkowalski/skill's; their
`name` fields and the two places that link to them (`ask-matt`, `break-ui`) say so. Nothing else was
changed from upstream, except `implement-spec`, which adds the **brief** (step 3) and its API pointers (step 5).
