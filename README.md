# Runesweeper

Minesweeper with the rules and quirks of the Windows classic, drawn on the GPU with
[Rune](https://github.com/anomalyco/rune): springs, ripples, liquid-metal chrome, plastic, fire
and shockwaves.

<p align="center">
  <img src="docs/play.gif" alt="Opening a field" width="330">
  <img src="docs/boom.gif" alt="Stepping on a mine" width="378">
</p>

<p align="center">
  <img src="docs/expert.webp" alt="Expert" width="100%">
</p>
<p align="center">
  <img src="docs/lost.webp" alt="A lost game: the mine burns and the rest come up" width="49%">
  <img src="docs/mono.webp" alt="Color off" width="49%">
</p>
<p align="center">
  <img src="docs/won.webp" alt="A won game" width="40%">
  <img src="docs/picker.webp" alt="The Mode and Level picker" width="40%">
</p>

## What moves, and what doesn't

Motion follows Emil Kowalski's rule of thumb: the more often you see something, the less it
moves. Every animation here had to justify itself (the `remove-animations` skill), and most
that couldn't were cut.

- **Every click** (hundreds a game): the tile lifts off in 240 ms, rippling out from the click
  over at most 300 ms. Numbers are there at once; nothing makes you wait to read them. The
  selector sits on the square under the pointer, with no glide to lag behind it.
- **Flags**: a 150 ms fade. **Keys** (arrows, Space, F, F2, Escape) never animate.
- **Panels** (the Level picker, Settings) are opened now and then, so they slide down from the
  toolbar in about 0.2 s; Escape closes one in the same frame.
- **A new field** from the face or the picker deals in from the centre in under half a second;
  from F2 it's simply there, old field gone in the same frame.
- **A loss**: one shockwave through the window and the other mines going off in a 300 ms chain
  out from the one you hit, then still.
- **A win** (rare, so it gets the flourish): flagged tiles turn gold, a swell runs across the
  field, stars burst off the face and the console glows. Beat your best and the timer turns gold.
- The result banner straddles the field's edge, says how it went, and leaves after 3 s.
- Shaders do the backdrop, the liquid-metal rim, the plastic face, the LED glow, the blasts and
  the glow, and hold still when nothing is happening: an idle window draws nothing.
- Entrances and exits use a strong ease-out; nothing grows from nothing. With Reduce Motion on,
  travel and bounce are off.
- Small windows and phones get the whole console, scaled to fit.

Motion is checked with Rune's motion gate (pops, jumps, snaps, overshoot that wasn't meant):

```sh
RUNE_MOTION_GATE=gate.txt runesweeper --scene beginner --script scripts/play.json --duration 4.5 --out frames
RUNE_MOTION_GATE=gate.txt runesweeper --scene lost --duration 4 --out frames   # also: keys.json, panels.json, won, picker
```

Every scenario reports nothing except the scripts that press a key with something to remove:
F2 in `play.json` and `keys.json`, where the one-frame swap shows up as the old field's tiles
leaving unanimated, and Escape in `panels.json`, where the picker leaves the same way. Both are
deliberate cuts for a keyboard action.

Frame cost stays small: `cargo run --release --example bench` times Rune's work per frame on
Expert (view, diff, layout, draw lists) at 0.4 ms median and 0.7 ms p95, even while a press is
dragged across the field. Nothing redraws at rest, and the shockwave's full-window backdrop copy
exists only for its 0.6 s.

## Solver

A bot plays every cell it can prove, then the hidden cell least likely to be a mine. On
Expert it wins **37.3%** of games (95% Wilson interval 36.4–38.3%), guessing 3.74 times per
win; 4.5% of games are won without a single guess; a game takes 1.6 ms.

The rules: Classic Expert (30×16, 99 mines), 10,000 fields dealt one after another from the
MSVC `rand()` stream seeded with 1, first click on the top-left cell. A mine there moves to
the first free cell as XP moves it, so the click is safe but need not open an area. This is
not the rule behind published "safe first click" figures (Tu et al.'s 40.06%, for one),
where the first cell's mine goes anywhere else on the field. The bot's odds assume every
layout equally likely and ignore the relocation's bias toward the top left.

No Guess draws uniform fields, with the 3×3 around a first click anywhere on the field kept
clear, until the bot clears one without guessing; there is no time limit and no mine is moved,
so an accepted field is a uniform draw over the fields logic can finish. Over 1,000 fields per
level (seed 1, release build, one native core):

| Level | Fields accepted | Generation p50 | p99 |
| --- | --- | --- | --- |
| Beginner | 89.9% | 0.08 ms | 0.27 ms |
| Intermediate | 70.7% | 0.42 ms | 1.8 ms |
| Expert | 15.3% | 5.3 ms | 31 ms |

The bot then wins every one of them without a guess.

Gentle No Guess is the same with the bot at Gentle strength: one number alone, two overlapping
numbers, and the count of mines left only once 12 or fewer cells are unresolved. Same rules:

| Level | Fields accepted | Generation p50 | p99 |
| --- | --- | --- | --- |
| Beginner | 88.5% | 0.04 ms | 0.13 ms |
| Intermediate | 69.5% | 0.34 ms | 1.3 ms |
| Expert | 13.2% | 5.3 ms | 35 ms |

The Gentle bot then wins every one of them without a guess.

```sh
cargo run --release --example solver -- 10000 1 1000   # Classic games per level, seed, fields per mode
```

It prints the same columns for Beginner and Intermediate; the same seed gives the same numbers.

## Download

Builds of the latest commit are on the [Latest build](https://github.com/nicognaW/runesweeper/releases/tag/latest)
release; tagged versions are on [Releases](https://github.com/nicognaW/runesweeper/releases).

| System | File | Install |
| --- | --- | --- |
| Windows 10/11 (x64) | `runesweeper-windows-x86_64.zip` | Unzip, run `runesweeper.exe`. SmartScreen may warn on an unsigned app: *More info → Run anyway*. |
| macOS 12+ (Apple silicon and Intel) | `runesweeper-macos-universal.dmg` | Open, drag Runesweeper to Applications. First launch: right-click the app → *Open* (it is not notarized). |
| Linux (x64) | `runesweeper-linux-x86_64.tar.gz` | `tar xzf runesweeper-linux-x86_64.tar.gz && ./runesweeper-linux-x86_64/install-linux.sh`, or run `./runesweeper` in place. Needs Vulkan and `libxkbcommon-x11`. |
| Web | [runesweeper.nkk.sh](https://runesweeper.nkk.sh), [nicognaw.github.io/runesweeper](https://nicognaw.github.io/runesweeper/) or `runesweeper-web.zip` | Needs WebGPU (recent Chrome or Edge; Safari 26+). Serve the zip's folder over HTTPS or `localhost`. |

## Play

The same game as Windows Minesweeper:

- **Left click** opens a square. Press and drag: the square under the pointer stays sunk until you let go, and the face gasps.
- **Right click** cycles flag → `?` → blank (`?` only with *Marks* on in Settings).
- **Keyboard**: arrows move the selector, Space opens (or chords on a number), F flags, F2 starts again, Escape closes a panel.
- **Touch** (the web build): the flag toggle on the toolbar makes taps place flags while it is lit; a tap on a number chords.
- **Chord**: on a number whose flags are all placed, press both buttons (or the middle one) to open its other neighbours. A wrong flag loses.
- Mines come from the Microsoft C runtime's `rand()`, seeded once per session from a millisecond clock, as `srand(GetTickCount())`: a column, then a row, each `rand() % n`, redrawn while the square already holds a mine.
- A first click on a mine moves it to the first free square reading from the top left, skipping the last row and column as Windows does. If that area is all mines, nothing happens and the next click is still a first click.
- The mine counter is mines less flags and goes negative. The timer starts on the first click, at 1, and stops at 999.
- Losing shows every mine, crosses out wrong flags and marks the one you hit in red. Winning flags the rest.
- **Modes**: Classic (原版) is everything above, unchanged. In **No Guess** (无猜) no mines exist until your first click; the field is then laid with the 3×3 around the click clear, so it always opens an area, and redrawn until perfect logic (the mine count included) can finish it from there. The field is fully determined by mode, level, seed and first click. The timer starts once the field is ready. **Gentle No Guess** (简单无猜) works the same, but only keeps fields that one number, or two overlapping numbers, at a time can finish, with the mine count near the end: pattern play always wins.
- **Toolbar**: the chip on the left shows the Mode and Level and opens the picker: a row for the Mode, then Beginner (9×9, 10), Intermediate (16×16, 40), Expert (30×16, 99) or Custom, whose width (9–30), height (9–24) and mines (up to (w−1)(h−1)) are typed in place; Enter or the Custom row starts it. Picking starts a new field at once.
- **Settings** (the button on the right): Marks (?), Color and Language (English or 简体中文). The first launch follows the system language (the browser's on the web): any Chinese locale means 简体中文. Until the MiSans font is bundled, Chinese uses the system's fonts on desktop, and may show as boxes on the web.
- The face starts a new game. `xyzzy`, then Shift, turns the old corner-pixel cheat on or off.

Settings are saved to `%APPDATA%\runesweeper`, `~/Library/Application Support/runesweeper`,
`~/.config/runesweeper`, or the browser's local storage.

`--plain` (desktop) or `?plain` (web) turns the shader effects off for weak GPUs.

## Build

Rust 1.97 or newer.

```sh
cargo run --release
```

Linux needs `libxkbcommon-dev libwayland-dev` to build. The web build installs what it needs
(the wasm target, the matching `wasm-bindgen` CLI, Rune's build script) and writes `dist/web`:

```sh
scripts/web-build.sh
```

### Headless

Every Rune app renders without a window, which is how the pictures above were made:

```sh
runesweeper --list-scenes
runesweeper --snap expert.png --scene expert --size 868x598 --scale 2
runesweeper --scene beginner --script scripts/play.json --video play.mp4
```

## Agent skills

`.claude/skills/` holds the design, animation and engineering skills agents use on this repo;
[`SOURCES.md`](.claude/skills/SOURCES.md) lists where each comes from. Two packs whose licenses
don't allow republishing are installed with `scripts/install-skills.sh`.

## Layout

- `src/game.rs`: the rules, with tests.
- `src/solver/`: proofs, exact odds, the bot and its benchmark (`examples/solver.rs`).
- `src/app.rs`: state, input (press, chord, flag), the clock, keys.
- `src/board.rs`, `src/ui.rs`, `src/icons.rs`, `src/style.rs`: the view, vector art and palettes.
- `src/toolbar.rs`: the toolbar and the panels it opens.
- `src/gfx.rs`: the compositor that turns effect slots into `rune-fx` shaders.
- `vendor/rune-fx`: Rune's effects crate with a one-line fix so it builds for wasm.

MIT. Rune is MIT, © Anomaly. Not affiliated with Microsoft.
