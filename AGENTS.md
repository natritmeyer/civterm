# civterm

Terminal Civilization game in Rust. Rust edition 2024, stable toolchain
(`rust-toolchain.toml` pins stable + clippy + rustfmt). Deps: crossterm,
ratatui, strum.

## Build and verify

The repo gate is `make build`:

```
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo build
cargo test
```

Run `make build` after any change. Current test baseline: 481 passing unit
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
  without revealing leaves its new surroundings in fog.
- A civilization is eliminated only by an actual loss, never by a sweep over
  start-of-game state: its last city captured (even with units still in the
  field — those are removed and the city-capture disband path covers
  homed units), or its last unit killed while it owns no cities. Test
  fixtures with zero units must NOT be eliminated. Eliminated players are
  skipped when the turn advances.

## TUI rendering invariants

- A city tile always wears its owning civilization's colour, even beneath an
  occupying unit, so a captured city flips colour the instant it falls
  rather than waiting for the victor to move off.
- Transient overlays (the battle flash) are painted in a final pass after
  tiles, markers and city-name labels, so they sit at the top of the
  z-order over everything the map draws.
- Wide (two-cell) glyphs such as 💥 must anchor in a tile's _left_ column; a
  wide glyph placed in the tile's right column spills a cell into the
  eastern neighbour. Tests assert the neighbour tile stays untouched.

## Tooling

- `scratch/` holds one-off refactor scripts (splitting/moving files). Keep
  them; they are not part of the build.
