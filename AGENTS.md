# civterm

Terminal Civilization game in Rust. Rust edition 2024, stable toolchain
(`rust-toolchain.toml` pins stable + clippy + rustfmt). Deps: crossterm,
ratatui, serde, serde_json, strum.

## Build and verify

The repo gate is `make build`:

```
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo build
cargo test
```

Run `make build` after any change. Current test baseline: 665 passing unit
tests. Keep this baseline line and the README's badge (`tests-N%20passing`)
in step with the actual count whenever tests are added or removed.

## Git policy

Never commit code — staging, committing, and pushing are the human's job.
Leave changes in the working tree; the human decides when and how to commit.

## Architecture

Top-level modules (`src/lib.rs`): `game_engine`, `model`, `tui`, `utils`.
The full design (module boundaries, command flow, turn loop, rendering
passes) is in `ARCHITECTURE.md`; on any conflict the specifics here win.

Layering is one-way: `game_engine` and `tui` depend on `model`; **`model`
must never reference `game_engine`** — no imports, no path mentions. This is
a hard boundary; keep it by convention.

## Code organization

- The facade files — `lib.rs` and top-level `mod.rs`s (`tui/mod.rs`,
  `game_engine/mod.rs`) — contain **only** `mod` declarations and `pub use`
  re-exports, never structural code. A module's *hub* `mod.rs` (a split
  module's own entry file, e.g. `tui/app/mod.rs`) instead hosts the primary
  type plus `mod` declarations and splits the concern-specific `impl` blocks
  into sibling files. This is a deliberate, strong preference.
- One concept per file. Implementation blocks are split by concern into
  sibling files rather than allowed to grow.
- `src/game_engine/` layout:
    - `mod.rs` — thin facade: `mod` + `pub use` (re-exports `Engine`,
      `GameView`, `Command`, `Player`, `Event`, `MoveError`, `SettleError`,
      `Exploration`, `CityIncome`, `DEFAULT_MAP_WIDTH/HEIGHT`).
    - `engine.rs` — `Engine` struct, `impl Default`, inherent `Engine` impl;
      owns `DEFAULT_MAP_WIDTH/HEIGHT`, `DEFAULT_SEED`. Fields are
      `pub(crate)`.
    - `engine_view.rs` — `impl GameView for Engine` (private module; the trait
      itself lives in `game_view.rs`).
    - `calendar.rs` — `CALENDAR_SCHEDULE` + `calendar_year` (`pub(crate)`).
    - `movement.rs`, `combat.rs`, `cities.rs`, `diplomacy.rs`, `research.rs`,
      `turns.rs` — private modules, one `impl Engine` block each by concern
      (`combat.rs` owns `HIT_POINTS`).
    - `rival_player_engine.rs` — `RivalMotion`, `RivalWar` (re-exported as
      `game_engine::RivalMotion`, `game_engine::RivalWar`) plus the rival AI:
      `run_rival_turn`, `drain_rival_motion`, `drain_rival_wars`, and the
      research/production/settle/garrison/warfare helpers. The AI's decisions
      are deterministic — it never consults `self.rng`; only the combat a
      fight initiates is resolved by the shared RNG.
    - `city_income.rs` — `CityIncome` (re-exported as
      `game_engine::CityIncome`, not a path under `game_view`).
    - `engine_tests.rs` — `#[cfg(test)] mod engine_tests;` in `mod.rs`.
- Keep `game_engine/` flat — every file directly in the directory, no
  subdirectories.
- Larger TUI modules live in hub + concern-sibling directories rather than
  one file per module:
    - `app/mod.rs` — the `App` struct, phases, dialog state types, the
      event loop and drawing, plus `mod` declarations. Sibling `impl App`
      files: `setup.rs` (startup panels), `playing.rs` (playing-phase key
      commands), `mouse.rs` (clicks, drags, hover), `dialogs.rs` (floating
      dialogs), `pickers.rs` (production/work pickers); tests in `tests.rs`.
    - `game_screen/mod.rs` — `GameScreen`, `BattleAnimation`, the pane
      draw methods (`draw_main_map`, minimap, stats, focus, event log) and
      the `Widget` impl. `tiles.rs` holds the per-tile painting helpers
      (`paint_tile`, `draw_city_label`, `civilization_color`, `tile_style`);
      tests in `tests.rs`. The hub re-exports `tiles` items it needs
      (e.g. `pub(crate) use` in the module for cross-file access).
- Other top-level modules follow the same pattern: concept per file,
  colocated tests.

## Conventions

- Sibling module files start with `use super::*;` plus their own explicit
  imports. Never rely on a sibling's private items crossing module
  boundaries — methods used across files need `pub(super)` (or `pub(crate)`
  where the layout demands it).
- Tests are colocated as `#[cfg(test)] mod tests;` in the module's
  `mod.rs`, the body in a sibling `tests.rs`. Tests in the tree may use
  field access and private/crate-internal APIs, so don't widen visibility
  unnecessarily. `#[cfg(test)]`-only items need `#[cfg(test)] use ...` so
  non-test builds stay warning-free (clippy runs with `-D warnings`).
- `///` doc comments carry the reasoning on constants and non-obvious
  functions; when a function moves, its doc comment moves with it. `///` is
  only legal on items — on a statement it hard-fails clippy
  (`unused_doc_comments`), so use `//` there.
- Don't alias modules that merely group implementation back together
  (`engine/engine.rs`), and don't add `#[allow(clippy::module_inception)]`
  to make bad nesting pass lint.

## Movement and reveal invariants

- Every code path that lands a unit on a new tile must be a mirror of the
  plain-move tail: spend the terrain cost (`spend_moves(cost)`, never
  `spend_turn()`), then `game.reveal_tiles_at(owner, destination)`.
  Combat advances and city captures are separate early returns in
  `move_unit`, so each reveals for itself — a unit that changes tiles
  without revealing leaves its new surroundings in fog. Boarding and
  disembarking are the one exception to the cost: both reveal, neither
  spends movement (the tail skips `spend_moves` when the mover was
  transported, and `try_board` never calls it).
- A civilization is eliminated only by an actual loss, never by a sweep over
  start-of-game state: its last city captured (even with units still in the
  field — those are removed and the city-capture disband path covers
  homed units), or its last unit killed while it owns no cities. Test
  fixtures with zero units must NOT be eliminated. Eliminated players are
  skipped when the turn advances.
- A unit's home city is `Option<CityId>`, never a placeholder id. A starting
  settler founded nothing and is honestly `None`; only a city-raised unit is
  `Some(city_id)`. This matters because a placeholder would collide with the
  first real city founded (ids are allocated from 0), which both listed every
  rival's settler in the new city's unit panel and made the conquest disband
  sweep dissolve unrelated civilizations' units along with the fallen city.

## Conquest

- A city falls to the attacker the moment its guard breaks, in either path:
  an at-war unit walking onto an undefended city tile, or winning combat on
  a defended city tile. Ownership transfers immediately, so the conquered
  city's tile wears the victor's colour the instant it falls — beneath the
  conquering unit, before it ever moves off.
- A conquered city of population one is destroyed outright: it is removed
  from the game and the tile reverts to plain terrain. Only a city grown to
  size two or more is captured and keeps producing for its new owner. Either
  fate disbands units homed to the fallen city, and a civilization left with
  no city at all is eliminated (its stragglers in the field disband with it).

## Rival AI and turn resolution

- `Command::EndTurn` resolves the whole round on the engine: each rival
  civilization takes its turn (`run_rival_turn` — research, production,
  settle, garrison), then control returns to the human and the turn number
  increments once. Rivals act in player order and eliminated players are
  skipped; the human's own turn begins again fresh (units' moves restored).
- A rival's settler picks its site (`best_settlement_site`) in tiers, every one
  measured against the union of **all** civilizations' 21-tile working grids —
  its own, its fellow rivals' and the human's — so no rival crowds itself,
  another rival, or the player: (1) open, sharing at most `CITY_FOOTPRINT_MAX_OVERLAP`
  (3) tiles (≥ Chebyshev 4 off every city); (2) acceptable, sharing at most
  `CITY_FOOTPRINT_ACCEPTABLE_OVERLAP` (6) tiles (≥ Chebyshev 3 out); (3)
  frontier — with only crowded sites visible the settler marches to the nearest
  explored land tile bordering unexplored ground (`nearest_frontier_edge`) so
  its reveals open up new country instead of founding in its own lap; (4)
  crowded — best-scored tile, kept only for a fully explored map (so the rival
  still fills a quiet continent rather than stall). Within every tier the pick
  is deterministic: best score first, then the site nearest the settler, then
  the smallest tile.
- A settler never enters the crowd unless the map gives no alternative: a
  site is only ever chosen from explored tiles, so the frontier push keeps the
  rival's cities spreading outward while respecting every working grid.
- Rival movement is recorded per step as `RivalMotion { unit, from, to,
  battle }` whenever a rival unit lands on a new tile — **every** way it lands:
  the plain move tail, boarding, a city capture and an attack alike, all through
  the single `movement::record_rival_step` helper, gated `owner != human` so the
  player's own moves are never recorded. A combat landing is recorded *before*
  the fight resolves, because an attacker repelled there is about to leave the
  game and the TUI still needs the landing to replay the attack. The TUI drains
  the record (`drain_rival_motion`, oldest first) after an EndTurn to replay it;
  the record is never persisted.
- A rival fights only fights it expects to win: it declares war on the human
  when one of its military units stands beside (any of the 8 neighbours, wrap
  included) a human unit or city whose strongest defender its power outguns
  (`attacker_power > defender_power`; an undefended city counts as a trivial
  win), and it never attacks below `RIVAL_MINIMUM_WIN_CHANCE` (10%).
  War is faction-wide: the first winnable encounter draws the whole rival into
  it, and other units may then attack freely on the same and later turns.
  Every declaration is recorded as a transient `RivalWar { rival }` that the
  TUI drains (`drain_rival_wars`, oldest first) after an EndTurn and announces
  with an OK window; the record is never persisted.
- The AI's decisions are deterministic: it never consults `self.rng` when
  choosing what to research, build, settle, march or fight — it sizes up
  powers, not luck. The one place a rival's action touches `self.rng` is the
  ordinary combat resolution shared with the player: when a rival attacks, the
  fight is fought (and the RNG stream advanced) exactly as the human's would
  be. Every rival step still goes out through the ordinary `move_unit` path,
  so the movement/reveal/transport/combat invariants hold.

## Transport invariants

- A naval transport (trireme, sail, frigate) carries at most
  `UnitClass::carry_capacity()` (2) land units. Boarding is the only lawful
  land→water transition: a land unit on a tile one step (any of the 8
  directions) from a friendly transport with a free berth moves onto the
  carrier's tile. Disembarking is a plain move from the carrier's tile back
  onto an adjacent land tile. Neither costs movement.
- A transported unit has no map square of its own. `GameView::units_at`
  omits it, but `player_units` keeps it so it can be selected to disembark;
  `GameView::unit(id)` finds it regardless. `Game::sync_cargo` re-points
  every cargo unit's `location` to the carrier after each carrier move.
  `GameView::cargo_at` reports the cargo sharing a tile, and the info panel
  lists it marked `Aboard:` / `aboard`. Cargo stays out of `units_at` because
  the map painter takes a tile's first unit to pick the letter to draw, and a
  land unit's letter on a hull's tile would misreport the fleet.
- A transported unit is inert: it cannot move except ashore, fortify, work,
  stand sentry, or found a city. Cargo never fights — `select_defender`,
  `enemies_present` and `ensure_peaceful_passage` skip transported units.
- When a carrier is removed, its cargo must go too: combat calls
  `Game::disband_cargo_of`, and `disband_units_homed_to` sweeps cargo
  aboard a doomed ship. No unit may reference a missing carrier.

## City build-completion notice

- A city that finishes a unit or an improvement hands the TUI a
  `BuildComplete { city, city_name, target }`, pushed onto the transient
  `Engine.builds` by `process_cities` and drained by
  `drain_build_completions`. Only the human's cities are recorded: a rival
  city finishing a build is news, not a decision for the player. The name is
  snapshotted because the city itself may be gone by the time the window is
  answered.
- The windows are a **loop**, not a stack: every city that finished gets its
  own window, one after another, oldest first. `OK` pops the front and shows
  the next (closing the window once the queue empties); `New Order` pops it
  too and opens *that* city's window instead.
- `New Order` sets `BuildNoticeState::suspended`, which parks the queue: the
  notice neither draws nor takes input, so the city window owns the screen.
  `close_city_window` calls `resume_build_notice`, so closing the city window
  returns to the loop at the next city rather than abandoning the rest.
  Suspending is not the same as dropping the queue — dropping it silently
  loses every city the player never saw announced.
- A suspended notice is therefore excluded from `window_is_open`, from the
  draw pass and from `handle_playing_key`; only a notice actually showing
  counts. Both the queue and its rectangle are cleared when a new game starts
  (`reset_setup`) and when the match ends (`check_game_over`), so a stale
  window can never follow the player to the menu or onto the victory screen.
- A round can advance a technology *and* finish a build. Research captures the
  keyboard and draws first, so the queue is parked (suspended) behind the
  research dialog and `research_dialog_confirm` resumes it — parking, not
  dropping, for the same reason `New Order` parks rather than drops.
- The notice is modal while it shows, exactly like the war-declaration window:
  `q` does not reach the quit path until the queue is answered.

## Save and load impact

New features are judged for their save impact before they are built:

- Ask first what state a feature introduces and whether it must survive a
  save/load round trip. Transient state — e.g. `Engine.motion` (the rival
  replay record), `Engine.builds` (the build-completion queue), the battle
  flash, UI-only selection — is rebuilt fresh (`Engine::into_loaded`
  re-initialises it) and must never be pushed into `SaveData`.
- Engine-level state is threaded by hand: `SaveData::capture` copies it and
  `SaveData::into_loaded` restores it. Adding a field to `Engine` (or to
  anything it owns that is not already inside `game: Game`) means updating
  both halves together — never let new engine state be silently dropped from
  `into_loaded`. Old fields must keep their `#[serde(default)]` so files
  written by older builds still load; a destructive reshuffle of the format
  bumps `SAVE_FORMAT_VERSION`.
- Model state (`Unit`, `City`, `Player`, `Game`) round-trips through the
  nested `game: Game` — a new field there needs `#[serde(default)]` (or a
  version bump) but no `capture`/`into_loaded` change, unless it references
  something outside its own struct (e.g. a unit's `aboard` carrier id), in
  which case the save round-trip test must confirm the reference still
  resolves after the load.
- Save impact is verified by tests, not left to luck: state new to a feature
  must appear in the round-trip tests (`a_captured_game_round_trips_through_json`
  and a targeted one where the state is non-trivial — e.g.
  `a_transported_unit_survives_save_and_load`), and the loader keeps refusing
  formats newer than it understands.

## TUI rendering invariants

- The left-hand info panel describes **every** unit on the focused or hovered
  tile, never just the first. A tile legitimately holds several units (a city
  garrison, or a second friendly unit stepping onto ground its own side already
  occupies), so both blocks iterate the whole list. Each unit takes its own row:
  joined onto one line a listing overruns the 36-column panel, and `draw_text`
  stops at the edge rather than wrapping, silently dropping every unit after the
  first. Every row is clamped to the panel's `bottom`, and a list too tall for
  the space reports `+N more` rather than overdrawing the map pane.
- Cargo appears in the tile info panel, marked `Aboard:` / `aboard`, so the
  manifest of a ship is visible without opening it. It stays out of
  `GameView::units_at`, which is what the map painter reads: a land unit's
  letter on a hull's tile would misreport the fleet. `cargo_at` reports it
  separately and the panel asks for both.
- A city tile always wears its owning civilization's colour, even beneath an
  occupying unit, so a captured city flips colour the instant it falls
  rather than waiting for the victor to move off.
- A unit fortifying on a city tile is hidden on the map: it has stowed itself
  as the city's garrison, so the tile keeps showing its population digit (and
  the city's name label) as if unoccupied. Any other unit still paints its
  letter ahead of the population.
- Transient overlays (the battle flash, the rival-move replay) are painted in
  a final pass after tiles, markers and city-name labels, so they sit at the
  top of the z-order over everything the map draws. The flash goes last of all,
  over the replayed rival glyph too, so an arriving attacker is eclipsed by the
  💥 it triggered.
- Wide (two-cell) glyphs such as 💥 must anchor in a tile's _left_ column; a
  wide glyph placed in the tile's right column spills a cell into the
  eastern neighbour. Tests assert the neighbour tile stays untouched.
- While the rival-move replay runs, units with frames still to play are
  lifted off their game-state squares (`hidden_units` fed through
  `paint_tile`) so only the overlay paints them — a unit "in transit" must
  not be left drawn at its destination. The replay plays each step as two
  300ms sub-phases (starting tile, then ending tile). A step flagged `battle`
  — one that ended in combat — erupts on arrival: the replay turns it into a
  `BattleAnimation` over the destination tile
  (`RivalMoveAnimation::active_battle`, whose clock starts at that frame's own
  onset so a fight late in a long round still flashes), and combat is always
  fought on the destination square, so the attacker marches off its own tile
  first. A step whose **every
  endpoint** is unexplored by the human is dropped at the source (the TUI
  filters the drained motion by the human's discovery map), so a rival
  wandering the fog stays hidden; a step with at least one explored endpoint
  is shown — including the from-fog-into-sight and out-of-sight-into-fog
  ends, where the glyph is drawn even over undiscovered fog (the glyph,
  never the terrain).

## Tooling

- Tests that start a game share one pinned world seed (`TEST_SEED` in
  `src/tui/app/mod.rs`, read by `App::new_game_seed` in `setup.rs`); a real
  game still draws a fresh clock-derived seed. A random map turns any
  assertion about terrain into a coin flip — a test wanting a passable tile
  beside the starting city failed on ~1.5% of maps, those that ring it with
  water and forest. If a fixture needs terrain the pinned world lacks, place
  the unit or tile explicitly rather than drawing a new map.

- `scratch/` holds one-off refactor scripts (splitting/moving files). Keep
  them; they are not part of the build.
