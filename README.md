# Runesweeper

Minesweeper with the rules, menus and quirks of the Windows classic, drawn on the GPU with
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
  <img src="docs/menu.webp" alt="The Game menu" width="40%">
</p>

## What moves

- A new field deals in from the centre; opened squares lift off in a ripple from the click, and
  their numbers pop up under a soft halo of their own colour.
- A highlight glides from square to square on a spring, and the face's eyes follow the pointer.
- A mine sends a shockwave through the window, knocks the console sideways, flashes it red and
  sets off the other mines in a chain outward from the hit; the one you stepped on burns.
- A win turns the flagged tiles gold, runs a swell across the field, throws sparkles and lights
  a glow around the console.
- The LED counters crossfade their segments like real LEDs. Shaders do the backdrop, the
  liquid-metal rim, the plastic face, the LED glow, ripples, fire and the shockwave.
- Nothing draws while nobody plays: the ambient effects ease to a stop 10 s after the last
  input and start again on the next. With Reduce Motion on, travel and bounce are off.
- Small windows and phones get the whole console, scaled to fit.

Motion is checked with Rune's motion gate (pops, jumps, snaps, overshoot that wasn't meant):

```sh
RUNE_MOTION_GATE=gate.txt runesweeper --scene beginner --script scripts/play.json --duration 4.5 --out frames
```

## Download

Builds of the latest commit are on the [Latest build](https://github.com/nicognaW/runesweeper/releases/tag/latest)
release; tagged versions are on [Releases](https://github.com/nicognaW/runesweeper/releases).

| System | File | Install |
| --- | --- | --- |
| Windows 10/11 (x64) | `runesweeper-windows-x86_64.zip` | Unzip, run `runesweeper.exe`. SmartScreen may warn on an unsigned app: *More info → Run anyway*. |
| macOS 12+ (Apple silicon and Intel) | `runesweeper-macos-universal.dmg` | Open, drag Runesweeper to Applications. First launch: right-click the app → *Open* (it is not notarized). |
| Linux (x64) | `runesweeper-linux-x86_64.tar.gz` | `tar xzf runesweeper-linux-x86_64.tar.gz && ./runesweeper-linux-x86_64/install-linux.sh`, or run `./runesweeper` in place. Needs Vulkan and `libxkbcommon-x11`. |
| Web | [nicognaw.github.io/runesweeper](https://nicognaw.github.io/runesweeper/) or `runesweeper-web.zip` | Needs WebGPU (recent Chrome or Edge; Safari 26+). Serve the zip's folder over HTTPS or `localhost`. |

## Play

The same game as Windows Minesweeper:

- **Left click** opens a square. Press and drag: the square under the pointer stays sunk until you let go, and the face gasps.
- **Right click** cycles flag → `?` → blank (`?` only with *Game ▸ Marks*).
- **Chord**: on a number whose flags are all placed, press both buttons (or the middle one) to open its other neighbours. A wrong flag loses.
- The first click is never a mine: one that was there moves to the top-left-most free square.
- The mine counter is mines less flags and goes negative. The timer starts on the first click, at 1, and stops at 999.
- Losing shows every mine, crosses out wrong flags and marks the one you hit in red. Winning flags the rest.
- **Game** menu: New (F2), Beginner (9×9, 10), Intermediate (16×16, 40), Expert (30×16, 99), Custom (9–24 × 9–30, up to (h−1)(w−1) mines), Marks, Color, Best Times, Exit.
- **Best Times** keeps the fastest name per level; beat one and you're asked for your name.
- The face starts a new game. `xyzzy`, then Shift+Enter, turns on the old corner-pixel cheat.

Settings and best times are saved to `%APPDATA%\runesweeper`, `~/Library/Application Support/runesweeper`,
`~/.config/runesweeper`, or the browser's local storage.

`--plain` (desktop) or `?plain` (web) turns the shader effects off for weak GPUs.

## Build

Rust 1.97 or newer.

```sh
cargo run --release
```

Linux needs `libxkbcommon-dev libwayland-dev` to build. The web build uses Rune's script
(clone Rune at the rev in `Cargo.toml`):

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version <version in Cargo.lock> --locked
path/to/rune/scripts/web-build.sh runesweeper --out dist/web --no-html && cp web/index.html dist/web/
```

### Headless

Every Rune app renders without a window, which is how the pictures above were made:

```sh
runesweeper --list-scenes
runesweeper --snap expert.png --scene expert --size 868x598 --scale 2
runesweeper --scene beginner --script scripts/play.json --video play.mp4
```

## Layout

- `src/game.rs`: the rules, with tests.
- `src/app.rs`: state, input (press, chord, flag), the clock, keys.
- `src/board.rs`, `src/ui.rs`, `src/icons.rs`, `src/style.rs`: the view, vector art and palettes.
- `src/gfx.rs`: the compositor that turns effect slots into `rune-fx` shaders.
- `vendor/rune-fx`: Rune's effects crate with a one-line fix so it builds for wasm.

MIT. Rune is MIT, © Anomaly. Not affiliated with Microsoft.
