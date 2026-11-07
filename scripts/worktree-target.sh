#!/usr/bin/env bash
# Gives a git worktree its own Cargo target dir that reuses the main checkout's dependency builds.
#
#   scripts/worktree-target.sh <worktree>
#
# Worktrees must not share one target dir: Cargo keys this crate's builds by package, not by
# checkout path, so worktrees overwrite each other's test binaries and MiSans subset, and a test
# can pass against another branch's code. This hard-links the main target dir into the worktree
# (no extra disk), then drops the path crates' artifacts, which Cargo rewrites in place and would
# corrupt through the links. The worktree rebuilds only those crates.
set -euo pipefail
wt=$(cd "$1" && pwd)
main=$(git -C "$wt" worktree list --porcelain | sed -n '1s/^worktree //p')
[ "$wt" != "$main" ] || { echo "run this on a worktree, not the main checkout" >&2; exit 1; }
[ -d "$main/target" ] || { echo "no $main/target to reuse" >&2; exit 1; }
[ ! -e "$wt/target" ] || { echo "$wt/target already exists" >&2; exit 1; }
cp -al "$main/target" "$wt/target"
crates=$(cd "$main" && cargo metadata --format-version 1 --offline 2>/dev/null |
    python3 -I -c 'import json,sys; [print(p["name"]) for p in json.load(sys.stdin)["packages"] if p["source"] is None]')
for c in $crates; do
    find "$wt/target" \( -name "*$c*" -o -name "*${c//-/_}*" \) -prune -exec rm -rf {} +
done
find "$wt/target" \( -name .rustc_info.json -o -name .cargo-lock \) -delete
echo "$wt/target: dependencies shared with $main/target; path crates ($(echo $crates)) rebuild here"
