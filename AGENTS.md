## Agent skills

### Issue tracker

GitHub Issues in `nicognaW/runesweeper`. See `docs/agents/issue-tracker.md`.

### Triage labels

Default labels: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: root `GLOSSARY.md` + `docs/adr/`. See `docs/agents/domain.md`.

## Git workflow

No pull requests. Every session, including Claude sessions, merges its changes into `main` and pushes `main`. If a session works on another branch, it merges that branch into `main` before finishing.
