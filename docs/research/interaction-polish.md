# Interaction polish from PR #1

PR #1 (`nkai/interaction-polish`, 5c2fe93) brought Minesweeper Noir's lighting and reveal details
to a `main` 85 commits older than today's. It was reviewed on two axes (standards, spec) and its
compatible parts ported by hand; the PR itself does not merge.

## Taken

| Change | Where |
| --- | --- |
| A soft lamp follows the pointer on a render-clock spring, without re-running the view; jumps over two cells snap | `app::Lamp`, `board::view` |
| An opened cell's cover goes in the frame it opens; numbers (digit and halo) never fade or move, the win's sweep included | `board::view`, `board::number` |
| A 250 ms pool of light under the pointer's opens, rippling from the click; the last four turns keep theirs, so a quick next click never cuts one short | `App::lit_turns`, `board::reveal_light` |
| Keys open and flag at once with no light, and put the lamp out | `App::key`, `App::step` |
| The loss's heat cools in 1.2 s, and the compositor keeps drawing until then, then sleeps | `gfx::HEAT_FOR` |
| Reduce Motion leaves out the lamp, shockwave, blasts, heat, stars and aura | `board::view`, `ui::view` |
| The deliberate cuts are declared on their elements (`MotionIntent::CUT`), so the motion gate stays clean without an allowlist | `board.rs` |

## Changed on the way

- A keyboard win or loss keeps its celebration or blast. The PR tied every effect to the last
  input being the pointer; only the reveal light is.
- Reveal waves are trimmed to the last four turns instead of accumulating for the whole game, and
  a keyboard open no longer unmounts waves still running.
- The PR's number fix set `Transition::none()` on the number's root only; its halo and digit still
  faded in over about 170 ms. Both now land in the frame.
- A review's step forward lights the cells it opens, where it used to lift their tiles.

## Left out

Obsolete against [spec.md](../spec.md) or out of scope: the 800 ms delay before the name prompt and
its dialog text (no name prompt or best times now), the reserved result line (the banner carries
Review), the About button and Game menu (the toolbar replaced them), removing How to Play and the
first-run hint (already gone), and the face's press scale.

## Checked

`cargo test --locked` (196), `cargo clippy --all-targets --locked -- -D warnings`, and the motion
gate on `play.json`, `keys.json`, `won` and `lost` (no reports) under Mesa's software Vulkan.
