# Gameplay and UX: Runesweeper vs Windows XP Minesweeper

Checked against Runesweeper at commit `78a6131`, and against XP `winmine.exe` as reconstructed by repnz ([ReversingMinesweeper](https://github.com/repnz/ReversingMinesweeper) @ `9981360`, files `Minesweeper/game.c`, `windowing.c`, `config.c`, `drawing.c`, `sound.c`, `*.h`). The Rune framework was read at rev `781e2b6` for event timing. Researched 2026-11-06. Board generation, the RNG and first-click relocation are covered in [minesweeper-generation.md](minesweeper-generation.md) and are not repeated here.

Repo citations are `src/<file>:<line>`. repnz citations are `game.c:<line>` and so on.

**About the source.** The reconstruction has some slips where it contradicts itself or would make the game unplayable. Examples: an inverted `STATE_GAME_IS_ON` test (`windowing.c:501`, `game.c:384`), a whole-byte compare against `BLOCK_STATE_FLAG` (`game.c:500`), a flag count compared without masking the revealed bit (`game.c:493`), and the negative counter digits (`drawing.c:206-207`). Where a finding depends on one of those lines, this document reads the intent, not the literal code, and says so.

> **Update:** fixed: #3 (a wrong chord opens the safe squares and reds every uncovered mine), #5 (Shift+left chords), #6 (the record box offers the current holder's name; blank stays blank; 32 characters), #7 (Marks off keeps ? marks), #9 (Shift alone toggles xyzzy; keys after the word are ignored; the pixel keeps its colour off the field), #10 (Custom opens with the current field), #11 (any first release on the field starts the clock). Open: #1, #2 and #4 need Rune to report the right button's press and window minimizing; #8 (Sound), #12 (one-click bug) and #13 (save on exit, window position) are left as they are.

## Verdict

The rules match XP closely: cell states, the flag/?/blank cycle, ? cells that can be opened, flags that block opening, the flood fill, chording only when the flag count equals the number, what a loss and a win reveal, the counter, the timer starting at 1 and capping at 999, the best-time rules, the menus and F2. The real differences, most player-visible first:

1. **Two-button chord depends on which button comes up first, and shows no 3×3 preview until then.** Rune runs the right-button handler on *release*. The chord registers only if the right button is released while the left is still held. If the left comes up first (or both at once), the click is a plain left click: nothing on a number, and an **open** on a hidden cell. XP chords on the first release of either button, shows the 3×3 preview as soon as the second button goes down, and a two-button press on a hidden cell does nothing.
2. **Right-click flags on release, not on press.** XP flags on `WM_RBUTTONDOWN`. In the repo the flag goes on the cell under the pointer at release, and only if the release is on the board.
3. **A chord with a wrong flag ends differently.** XP still opens the safe cells in the 3×3 and draws *every* unflagged mine there with the red background. The repo opens nothing and shows one red mine.
4. **The timer doesn't pause while the window is minimized.** XP stops counting when minimized and resumes on restore. The repo counts wall-clock time.
5. **Shift + left click doesn't chord.** XP treats Shift+left exactly like both buttons.
6. **The New Record name box starts as "Anonymous".** XP pre-fills the level's current record holder's name, which is usually your own. XP saves a blank name as blank; the repo saves it as "Anonymous".
7. **Turning Marks off clears the ? already on the board.** XP only changes what the next right-click does.
8. **No Sound option.** XP has *Game ▸ Sound* (off by default), with a tick each second and win and lose sounds.
9. **xyzzy details differ.** XP toggles the cheat with Shift alone, at the desktop's (0,0) pixel. The repo needs Shift+Enter and draws in the window's corner. Smaller differences are in the details below.
10. **Custom dialog.** XP fills it with the *current* board's size and mines. The repo fills it with the last custom values. Per repnz, XP's Cancel still switches the level to Custom.
11. **The first release that does nothing doesn't start the clock.** In XP, any release on the board with nothing yet revealed starts the timer, even on a flag or a two-button press on a hidden cell. In the repo, only a release that opens a cell starts it.
12. **No one-click bug.** This is already listed in the generation document.
13. **Persistence details.** XP saves on exit and remembers the window position. The repo saves straight away and doesn't keep the position.

Everything else checked matches, or is an addition that doesn't change the rules: animations, the result banner, keyboard play, touch flag mode, the hover selector, the first-run hint, the How to Play dialog, a resizable window that scales the console, Cmd+N/Q/W, and Escape to close a menu.

## Comparison

| Axis | XP (repnz) | Repo | Match? |
|---|---|---|---|
| Open | Left **release** on the board (`WM_LBUTTONUP` → `ReleaseMouseCapture` → `HandleBlockClick`, `windowing.c:482-487`, `game.c:428-464`) | Drag `Up` → `release` (`src/app.rs:308-315`) | Yes |
| Press preview | Unrevealed, unflagged cell sunk; ? shows the "pressed ?" sprite; flags stay up (`game.c:178-192`, `273-287`) | `pressed` keeps Hidden/Question (`src/board.rs:41-53`) | Yes |
| Drag moves preview / release outside cancels | Yes, through capture; release outside the board does nothing (`game.c:253-271`, `429`) | `Move` updates `press.cell`; `cell None` → return (`src/app.rs:300-307`, `204`) | Yes |
| Press after game over | Ignored (`windowing.c:543-545`) | Ignored (`src/app.rs:291-293`) | Yes |
| Right click | Toggles on **press** (`WM_RBUTTONDOWN` → `HandleRightClick`, `windowing.c:506-532`) | Rune's `on_secondary_press` fires on **release** over the same element (Rune `runtime/input/secondary.rs:35-50`); `src/board.rs:187`, `src/app.rs:319-331` | **No** (#2) |
| Both buttons | Right down while left captured → 3×3 mode at once (`windowing.c:516-520`); left down with right held → 3×3 (`windowing.c:547`); chord on the first button up (`windowing.c:482-487`) | `press.right` set only on right *release* while left is held (`src/app.rs:323-326`); chord decided at left release (`src/app.rs:309`) | **No** (#1) |
| Middle button | 3×3 press and chord (`windowing.c:495-505`) | `pointer_buttons([Middle])`, `press.middle` (`src/board.rs:182`, `src/app.rs:295`) | Yes |
| Shift + left | `Is3x3Click = wParam & (MK_RBUTTON\|MK_SHIFT)` (`windowing.c:547`) | Not handled | **No** (#5) |
| 3×3 preview | Every unrevealed cell in the 3×3, centre included (`game.c:194-247`) | Centre + neighbours, Hidden/Question (`src/board.rs:47-51`) | Yes (once it shows) |
| Chord rule | Centre revealed and flags in the 3×3 == its number, else nothing (`game.c:490-524`) | `src/game.rs:199-206` | Yes |
| Chord on hidden cell / on 0 | Nothing / nothing to open | `src/game.rs:200` | Yes |
| Chord, wrong flag | Every 3×3 cell processed: safe ones opened (and flooded), each unflagged mine red, then loss (`game.c:496-512`) | Loses on the first mine found, opens nothing (`src/game.rs:212-214`) | **No** (#3) |
| Flag cycle | blank → flag → ? (Marks on) or blank → blank (`game.c:402-413`) | `src/game.rs:220-238` | Yes |
| Marks toggle | Flips the setting only (`windowing.c:670-674`) | Also clears every ? (`src/app.rs:258-264`, `src/game.rs:240-247`) | **No** (#7) |
| ? cells | Can be opened (only flags are excluded, `game.c:455`) | `src/game.rs:172` | Yes |
| Flags block open / chord / flood | Yes (`game.c:455`, `355`) | `src/game.rs:172`, `207`, `253`, `265` | Yes |
| Flood | BFS from 0s; opens ? cells, skips flags (`game.c:345-376`, `588-615`) | `src/game.rs:249-273` | Yes |
| Mine counter | Mines − flags, can go negative; "-" then two digits (`drawing.c:190-217`) | `src/game.rs:165-168`, `src/icons.rs:145-148`: −99..999 | Yes (below −99: see details) |
| Timer start | On the first in-board release while nothing is revealed and the time is 0: shows 1 (`game.c:430-440`) | On a release once status is `Playing` (`src/app.rs:195-200`, `222`) | Mostly (#11) |
| Tick / cap | `SetTimer` 1000 ms; stops at 999 (`game.c:437`, `528-534`) | `floor(elapsed)+1`, `min(999)` (`src/app.rs:377`) | Yes |
| Stops on win/loss | `IsTimerOnAndShowed = FALSE` (`game.c:305`) | `started_at = None` (`src/app.rs:227`, `231`) | Yes |
| Minimized | Paused (`game.c:467-487`, `windowing.c:566-578`) | Keeps counting (`src/app.rs:373-384`) | **No** (#4) |
| Loss reveal | Unflagged mines (? too) black; wrong flags crossed; hit mine red (`game.c:126-151`, `583`) | `src/board.rs:113-123`, `136-138` | Yes |
| Win reveal | All mines flagged; counter to 0 (`game.c:311-315`) | `src/game.rs:292-304` | Yes |
| Faces | Normal, O (any capture), dead, sunglasses, pressed face (`drawing.h:4-8`, `windowing.c:745`) | Smile/Oh/Dead/Cool (`src/app.rs:171-178`); a pressed face scales down (`src/ui.rs:50`) | Yes |
| Face click | New game on a release over the face (`windowing.c:106-177`) | `on_press` → `new_game` (`src/ui.rs:54`) | Yes |
| Best time | Win, not Custom, strictly `<`, so ties don't count (`game.c:321-327`) | `src/app.rs:237-244` | Yes |
| Record dialog | Pre-filled with the current holder's name, 32 chars max, then Best Times (`windowing.c:241-301`) | Pre-filled "Anonymous"; blank → "Anonymous" (`src/app.rs:241`, `246-251`) | **No** (#6) |
| Best Times / Reset | Three rows; Reset sets 999/Anonymous, dialog stays open (`windowing.c:333-352`) | `src/ui.rs:207-222`, `src/app.rs:253-256` | Yes |
| Game menu | New, Beginner, Intermediate, Expert, Custom…, Marks (?), Color, **Sound**, Best Times…, Exit (`resource.h:27-36`) | Same without Sound (`src/ui.rs:97-133`) | **No** (#8) |
| F2 | New game (accelerator table) | `src/app.rs:426-432` | Yes |
| Level change | New game; window resized (`windowing.c:617-627`, `InitializeWindowBorder`) | `src/app.rs:163-169` | Yes |
| Custom dialog | Pre-filled from the current `GameConfig` H/W/M (`windowing.c:207-212`); Cancel still sets Custom (`windowing.c:883-889`) | Pre-filled from `saved.custom` (`src/app.rs:271-277`); Cancel closes | **No** (#10) |
| xyzzy | X,Y,Z,Z,Y key-downs, then Shift toggles; desktop pixel (0,0) (`windowing.c:699-738`, `749-773`) | "xyzzy" text, then Shift+Enter; 2×2 in the window corner (`src/app.rs:437-451`, `src/ui.rs:354-357`) | **No** (#9) |
| Window | Fixed size, sized to the board | Resizable, console scaled (`src/ui.rs:317`) | Addition |
| Persistence | Registry on exit if changed: level, H/W/M, Mark, Color, Sound, X/Y position, times, names (`config.c:53-76`, `windowing.c:876-878`) | File / localStorage on every change: level, custom, marks, color, played, best (`src/storage.rs:6-28`, `112`) | Mostly (#13) |

## Details

### 1. Two-button chord timing

- **XP.**
  - Left down starts a capture (`CaptureMouseInput`, `windowing.c:740-747`).
  - Right down during a capture clears the preview, sets `Is3x3Click`, and reposts a mouse move, so the 3×3 sinks at once (`windowing.c:516-520`).
  - Left down with the right already held enters 3×3 mode straight away, through `wParam & MK_RBUTTON` (`windowing.c:547`).
  - **Any** button-up while captured runs the click (`windowing.c:482-487`). The capture then ends, so the other button's release does nothing.
- **Repo.**
  - The board's right handler is `on_secondary_press_local` (`src/board.rs:187`). Rune only queues `SecondaryPress` from `secondary_up`, and only when the release is on the element that took the press (Rune `crates/rune/src/runtime/input/secondary.rs:35-50`).
  - So `press.right` becomes true only when the right button is *released* while the left is held (`src/app.rs:323-326`).
  - `pressed()` shows the 3×3 only when `press.chord()` is true (`src/board.rs:48`, `src/app.rs:39-41`).
- **Effect.**
  - Hold both buttons: you see a single sunk cell, not the 3×3.
  - Release the right first: the 3×3 appears, and the left release chords.
  - Release the left first, or both together: `release(false, …)` calls `game.open` (`src/app.rs:309-313`, `217`). On a number that does nothing. On a hidden cell it **opens it**, which XP never does from a two-button press, and that can lose the game. The right release then reaches `mark` on a cell that is now open, which does nothing.
  - "Right first, then left" (common in XP) has the same problem. In XP, the right press would also flag a hidden cell, but on a number it does nothing.
- **Middle-button chords are unaffected:** Rune's other buttons get `on_drag` Down/Up (`gesture.rs:170-206`).

### 2. Flag on release

- XP flags in `WM_RBUTTONDOWN` (`windowing.c:524-530`).
- In the repo the flag waits for the release, lands on the cell under the release point (`src/app.rs:328`), and is dropped if the release is off the board.
- The O-face doesn't show for a right press in either game (`src/app.rs:175`; XP takes no capture for a right press alone).

### 3. Chord with a wrong flag

- **XP** (`game.c:496-512`) loops over all nine cells. Non-mines get `ExpandEmptyBlock`, which floods from 0s and skips flags. Unflagged mines get `BLOCK_STATE_BOMB_RED_BACKGROUND`. `FinishGame(FALSE)` comes after the loop. So the safe cells open, and two unflagged mines in the 3×3 both show red.
  - The literal test `blockValue == BLOCK_STATE_FLAG` (`game.c:500`) would treat a *correctly* flagged mine as hit. That has to be a reconstruction slip, since chording would always lose. The reading here is "flagged, or not a mine".
- **Repo** (`src/game.rs:212-214`) returns `lose(m)` for the first mine in `targets` before opening anything, and only `hit` is drawn red (`src/board.rs:117-120`). The other unflagged mines show as ordinary loss mines.

### 4. Minimized timer

- **XP.** `SC_MINIMIZE` → `NotifyMinimize` saves `IsTimerOnAndShowed` and clears it while a game is on, and `TickSeconds` only counts while it is set (`game.c:467-479`, `528-534`). `SC_RESTORE` puts it back (`game.c:481-487`). Time spent minimized isn't counted.
  - Losing focus to another window doesn't pause it. Only minimizing does.
- **Repo.** `tick` computes `(now - t0) + 1` (`src/app.rs:377`), so time spent minimized is counted. Rune keeps polling while hidden, and in any case the arithmetic catches up on restore.

### 5. Shift-click chord

- `windowing.c:547`: `Is3x3Click = (wParam & 6) ? TRUE : FALSE;`, where 6 is `MK_RBUTTON | MK_SHIFT`.
- The repo's drag `Down` ignores modifiers (`src/app.rs:294-297`).

### 6. Record name

- **XP.** `SaveWinnerNameDialogProc` puts `GameConfig.Names[level]` in the edit box on `WM_INITDIALOG` and reads it back on any close (`windowing.c:250-298`). A returning player sees their own name. The default is "Anonymous", loaded from the registry or the string resource (`config.c:100-113`). An empty entry is stored empty.
- **Repo.** Pre-fills "Anonymous" (`src/app.rs:241`) and replaces an empty entry with "Anonymous" (`src/app.rs:248`).
- Both pass ties over (`<`) and record only Beginner, Intermediate and Expert.

### 7. Marks off

- **XP** `ID_MENUITEM_MARKS` only flips `GameConfig.Mark` (`windowing.c:670-674`). Existing ? stay, and the next right-click on one clears it (`game.c:407-409`).
- **Repo** clears them all (`src/app.rs:260-262`).

### 8. Sound

- **XP** has *Game ▸ Sound* (`ID_MENUITEM_SOUND`, `resource.h:33`; handler `windowing.c:675-686`). The registry default is 0 (off) (`config.c:128`). When on, it plays a tick on the first click and every second, plus win and lose sounds (`game.c:432`, `532`, `317`; `sound.c:22-43`).
- **Repo** has no audio and no menu item.

### 9. xyzzy

- **Typing.** XP compares raw `WM_KEYDOWN` codes against `"XYZZY"`, so case doesn't matter (`windowing.c:728-736`). A wrong key resets the count. Once the count reaches 5, other keys are ignored.
  - The repo matches text input (`src/app.rs:437-446`). Typing anything after the full word resets it before Shift+Enter, and non-text keys never reset it.
- **Toggle.** In XP, Shift key-down XORs the index between 5 and 17 (`windowing.c:701-706`), so Shift alone turns the cheat on and off, with no Enter.
  - Shift+Enter works too, because Shift goes down first. That is why the folk recipe works.
  - The repo needs Enter with Shift held (`src/app.rs:447-451`). It resets the index after toggling, so turning the cheat off means typing xyzzy again.
- **State 5.** At index 5 (word typed, no Shift yet), XP already sets the pixel on mouse moves unless `wParam == MK_CONTROL` (`windowing.c:756`). Folk descriptions say the pixel shows while Ctrl is held. Either way the literal condition is suspect: **unverified**.
- **Pixel.**
  - XP: `SetPixel` at desktop (0,0), black for a mine and white otherwise. It updates only on mouse moves inside the board while no button is captured. Moving off the board leaves the last colour (`windowing.c:756-772`).
  - Repo: a 2×2 square in the window's top-left. It turns black off the board, because `hover_cell` is `None` (`src/ui.rs:354-357`). It also updates during a press.
  - Drawing on the desktop isn't possible for a sandboxed or web app, so the window corner is a reasonable stand-in.

### 10. Custom dialog

- **XP.**
  - XP keeps no separate custom size. Choosing a preset overwrites `GameConfig.Height/Width/Mines` (`windowing.c:620-622`), and the dialog shows those values (`windowing.c:209-211`). On Expert it opens at 16 / 30 / 99.
  - Per repnz, `CustomFieldDialogBox` sets `Difficulty = 3` whatever the result, so Cancel also selects Custom (`windowing.c:883-889`, flagged "WIERD" by repnz).
  - It doesn't call `InitializeNewGame`. That may be a gap in the reconstruction (see open questions).
- **Repo.** Pre-fills `saved.custom` (`src/app.rs:272`). OK always starts a new custom game (`src/app.rs:279-285`). Cancel changes nothing.

### 11. Clock start on an empty release

- `HandleBlockClick` starts the timer whenever the release is in range and `NumberOfRevealedBlocks == 0 && TimerSeconds == 0`, before it checks what the click does (`game.c:429-440`).
  - A first left release on a flagged cell, or a first middle or two-button release, starts the clock.
- The repo's `start_clock` needs `status == Playing` (`src/app.rs:196`), which only `open` sets (`src/game.rs:176`).
- Both start the clock in XP's "nowhere to move the mine" case: the repo sets `Playing` before returning `Nothing` (`src/game.rs:176-183`).

### 12. One-click bug

- XP's relocation path never checks for a win (`game.c:578-580` vs `573`). See the generation document. The repo checks for a win on every open (`src/game.rs:195`).

### 13. Persistence

- **XP.**
  - It writes the registry only at exit, and only if `NeedToSaveConfigToRegistry` (`windowing.c:876-878`). Every menu change, record and reset sets that flag.
  - It saves the window `Xpos/Ypos` (`config.c:65-66`, updated on `WM_WINDOWPOSCHANGED`, `windowing.c:591-597`).
  - It also has an F5/F6 menu-bar toggle, gated on a registry `Menu` value that defaults to 0 (`windowing.c:718-727`, `config.c:131`). Players don't see it by default.
- **Repo.**
  - It saves on every change (`src/app.rs:166`, `220`, `249`, `255`, `263`, `268`).
  - It saves no window position.
  - `played` only drives the first-run hint (`src/storage.rs:11-12`, `src/ui.rs:383-385`).

### Minor and trivial

- **Counter below −99.** XP would show "−" and then `|n| mod 100`. The literal `LeftFlags % 100` (`drawing.c:207`) feeds a negative digit index, which is a slip; the reading here is `(-LeftFlags) % 100`. The repo clamps at −99 (`src/icons.rs:146`). Reaching it takes 100 more flags than mines, so only absurd over-flagging on Intermediate, Expert or Custom shows the difference.
- **Name length.** XP limits the name to 32 characters (`windowing.c:257-258`, `295`). The repo has no limit.
- **Activation click.** XP swallows the click that activates its window (`WM_ACTIVATE`/`WA_CLICKACTIVE` → `IgnoreSingleClick`, `windowing.c:586-590`, `496-498`, `507-510`, `535-538`). Whether Rune passes that click through wasn't checked.

### Additions (not mismatches)

- **Keyboard play.** Arrows, Space opens or chords, F flags (`src/app.rs:398-425`). Plus Cmd/Ctrl+N, Cmd+Q/W, and Escape to close a menu (`src/app.rs:433-462`).
- **Touch flag mode.** A tap flags, and a tap on a number chords (`src/app.rs:205-209`, `src/ui.rs:160-166`).
- **Visuals and help.** The hover selector, deal and ripple animations, the result banner, the gold timer on a new best, the first-run hint, How to Play, and `--plain`.
- **Window.** It can be resized, and the console scales to fit.

None of these change the rules.

## Open questions

- **Custom OK.** Does XP start a new game after Custom OK? repnz's `CustomFieldDialogBox` doesn't call `InitializeNewGame` (`windowing.c:883-889`). Retail XP appears to resize the board at once (**unverified**), so this is probably an omission in the reconstruction. Also unconfirmed: whether Cancel really leaves *Custom* checked.
- **xyzzy at index 5.** Is the pixel live after "xyzzy" alone, and does Ctrl enable it or suppress it? (`windowing.c:756` reads `!= MK_CONTROL`; folk sources say "hold Ctrl".)
- **Game menu order and accelerators.** The `.rc` file isn't in repnz's repo. The order above comes from the resource IDs and memory of XP: New / Beginner–Custom / Marks, Color, Sound / Best Times / Exit. Also unconfirmed: whether F2 is the only game accelerator, and whether F1 opens Help.
- **Flood queue overflow.** The flood queue is a 100-entry ring (`game.c:367-374`, `609-613`). Can a very large opening overrun it and leave cells unopened in XP? That would need a frontier of more than 99 pending zeros, which is unlikely on 30×24.
- **Activation click.** Does Rune deliver the window-activating click to the board? If it does, the repo differs from XP's `IgnoreSingleClick`.

## Sources

- repnz, ReversingMinesweeper (XP winmine reconstruction), commit `9981360`: https://github.com/repnz/ReversingMinesweeper (`Minesweeper/game.c`, `game.h`, `windowing.c`, `config.c`, `config.h`, `drawing.c`, `drawing.h`, `sound.c`, `resource.h`)
- Rune at rev `781e2b6d` (event timing): `crates/rune/src/runtime/input/secondary.rs`, `gesture.rs`, `pointer.rs`
- Secondary sources weren't needed for the findings above. Statements marked **unverified** rest on memory or folk descriptions.
