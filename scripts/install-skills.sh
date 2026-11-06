#!/usr/bin/env bash
# Installs the agent skills this repo uses but may not republish, into .claude/skills
# (git-ignored), from the commits the vendored set was reviewed against:
#   - Jakubantalik/transitions.dev: transitions-dev, transitions-polish, transitions-ui-generation.
#     Its license lets you install and use them, but not redistribute the collection.
#   - jnsahaj/skills: the whole pack. It has no license file, so it isn't vendored.
# Everything else in .claude/skills is committed; see .claude/skills/SOURCES.md.
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
dest="$root/.claude/skills"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

fetch() { # repo commit dir
    git init -q "$tmp/$3"
    git -C "$tmp/$3" fetch -q --depth 1 "https://github.com/$1" "$2"
    git -C "$tmp/$3" checkout -q FETCH_HEAD
}

fetch Jakubantalik/transitions.dev e3aefb554d8fd2cb3e91e773e82a0f2d5a4f82a0 transitions
for s in transitions-dev transitions-polish transitions-ui-generation; do
    rm -rf "$dest/$s"
    cp -R "$tmp/transitions/skills/$s" "$dest/$s"
    echo "installed $s"
done

fetch jnsahaj/skills 170b0b9192842559d1628c85e8bfe53c1bab61eb jnsahaj
for s in code-refactor-review create-draft-pr explain-then-fix ga ga-pr pit-of-success prepare-branch-context ux-flow-plan zero-tech-debt ; do
    rm -rf "$dest/$s"
    cp -R "$tmp/jnsahaj/skills/$s" "$dest/$s"
    echo "installed $s"
done
