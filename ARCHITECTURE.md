# civterm — Architecture

A terminal implementation of a classic turn-based civilization game in Rust
(edition 2024). This document describes how the program is put together; the
working conventions for editing it live in `AGENTS.md` and win on any
conflict.

## Layers and the one-way dependency rule

The crate is split into four top-level modules (`src/lib.rs`):

```
              ┌──────────┐   Command    ┌─────────────┐   GameView    ┌─────────┐
  keystrokes  │   TUI    │ ───────────▸ │ game_engine │ ◂───────────  │   TUI   │
  / mouse ──▸ │ (menus,  │   (mutations)│  (simulation)│  (read-only) │ (render)│
              │  dialogs)│ ◂─────────── │             │   view       │         │
              └──────────┘    Events    └──────┬──────┘               └─────────┘
                                               │ owns / mutates
                                         ┌─────▼─────┐
                                         │  model    │  (pure data + rules)
                                         └───────────┘
```

- `model` — the world: tiles, map, units, cities, civilizations, terrain,
  advancements. Pure data and rules with no I/O, no RNG, and no awareness of
  the game loop. It must **never reference `game_engine`** — no imports, no
  path mentions. This is the hard boundary of the project.
- `game_engine` — the simulation. Owns the whole game state plus its own
  turn counter, current player, event log and RNG. Two interfaces:
    - `Engine::submit(Command) -> Vec<Event>` — the only way anything changes
      the world. The TUI never mutates game state directly.
    - `GameView` (implemented by `Engine`) — the read-only view the renderer
      draws from: tiles, units, cities, income, research, calendar year.
- `tui` — the terminal UI: menu/setup flows, dialogs, and the map renderer.
  It sends `Command`s and consumes returned `Event`s; it reads world state
  only through `GameView`.
- `utils` — shared plumbing (`Rng`, `random_seed`), free of game concepts.

## Command / event flow

Every action is a `Command` (`game_engine/command.rs`): move, fortify, work,
found a city, set production, declare war, make peace, set a research target,
or end the turn. `Engine::submit` dispatches to the concern-specific
implementation (`movement.rs`, `combat.rs`, `cities.rs`, `diplomacy.rs`,
`research.rs`, `turns.rs`), appends human-readable `Event::new(...)` messages
to its internal log, and hands the batch back for the UI to display.

The turn loop (`turns.rs`):
1. `end_turn` resolves the **whole round** back to the human: each rival
   civilization takes its own turn in player order (`run_rival_turn` in
   `rival_player_engine.rs` — research, production, settle, garrison), then control returns
   to the human and the global `turn` increments once. The human's turn then
   begins again fresh (`begin_turn` restores moves, or advances frenetic
   terrain work and applies finished improvements, then processes the human's
   cities and research).
2. The rival AI is deterministic — it never draws from `self.rng`, so the
   combat RNG stream is untouched — and every rival step goes out through the
   ordinary `move_unit` path, so it respects the movement/reveal/transport/
   combat invariants.
3. Rival unit landings are recorded per step as `RivalMotion { unit, from,
   to }` (the plain move tail and boarding only, gated `owner != human`). The
   TUI drains the record after an EndTurn and replays it as animation; the
   record is never persisted or saved.

Eliminated civilizations are **skipped**, never swept; a player is eliminated
only by an actual loss (last city captured, or last unit killed while owning
no cities).

## World representation

- `model/cartography/` — `Map` (a `Vec<Vec<Tile>>`), `Tile` (terrain +
  improvement + resource), `Location`, `Direction`, and the `MapGenerator` in
  `cartography/generation/`. The world wraps east/west and clamps north/south.
- Entities use small index-backed ID types (`UnitId`, `CityId`, `PlayerId`)
  resolved by linear scan against flat `Vec`s — no hash maps, no RNG-backed
  UUIDs. Ownership runs `unit/home_city` → `city` → `player`.
- Every player keeps a fog-of-war bitmap of explored tiles. A unit landing on
  a tile reveals a radius-1 neighbourhood (`Game::DISCOVERY_RADIUS`); cities
  reveal their 21-tile working footprint. **Any** code path that moves a unit
  to a new tile reveals for the owning player — plain moves, combat advances,
  and city captures alike.

## Calendar and determinism

- `calendar.rs` maps a turn number to a displayed year via a fixed era
  schedule that slows as eras pass (BC is negative; there is no year 0).
- RNG is a seeded value object (`utils::rng.rs`). `Engine::with_seed` makes
  the whole game reproducible; `Engine::new_random` seeds from the system
  clock. The map, the starting positions, and every later random draw come
  out of that stream.

## Save and load

- `game_engine/save.rs` serializes the whole match — engine (map, units,
  cities, diplomacy, research, turn counter, current player, RNG) plus the
  UI-level competition and difficulty and the engine's event log — to JSON
  via serde, one format per version (`SAVE_FORMAT_VERSION = 1`). The year is
  derived from the turn on load, never stored. A newer save than the running
  build understands is refused with a version error.
- The `'S'` key in play opens a textual prompt (default path `civterm.civ`);
  "Load Saved Game" on the splash menu does the same for loading. The prompt
  is keyboard-only and consumes the mouse while open. The typed path is
  handed to `std::fs` exactly as given.

## Rendering

`main.rs` puts the terminal into raw mode, an alternate screen, and mouse
capture (including any-motion tracking for hover), then runs the `App` event
loop. `App` (`tui/app/`) owns the `Engine` plus UI-only selection state
(focused unit/city, camera, dialog state). Each frame it constructs a
`GameScreen` against the engine's `GameView` and draws it as a ratatui
`Widget`.

`game_screen/` paints the screen in passes, in z-order:
1. Per-tile painting (`tiles.rs`) — the map pane, tile styles and terrain
   colours, units, city markers. While a rival-move replay runs, units whose
   frames are still to play are lifted off their game-state squares
   (`hidden_units` fed through `paint_tile`) so only the overlay paints them.
2. City-name labels — collected during the tile pass, drawn afterwards so a
   label survives the rows below it.
3. Final-pass overlays — transient effects (the battle explosion flash, the
   rival-move replay) sit on top of everything the map drew.

The rival-move replay (`game_screen/mod.rs`) plays each recorded step as two
300ms sub-phases — on the starting tile, then the ending tile. Only steps the
human can trace are played: the drained motion is filtered by the human's
discovery map, so a step whose **every** endpoint is unexplored is dropped and
a rival marching through the fog stays hidden; a step with at least one
explored endpoint is shown, painting the moving unit in its owner's colour
even over undiscovered fog (the glyph, never the terrain). While it plays,
the `App` swallows game input (modals and dialogs still capture) and pans the
camera when the active step leaves the central 70% of the viewport.

Map panes other than the main map (minimap, player stats, focus, event log)
and the whole map-render control flow live in `game_screen/mod.rs`.

Two rendering invariants worth repeating (full list in `AGENTS.md`):

- A city tile always wears its owning civilization's colour, even beneath an
  occupying unit — so a captured city flips colour the instant it falls.
- Wide (two-cell) glyphs such as 💥 anchor in a tile's **left** column, so
  they never spill into the eastern neighbour.

## Testing strategy

- Unit tests are colocated per module (`#[cfg(test)]`), bodies in a sibling
  file (`*_tests.rs` or `tests.rs`). They may use private/crate-internal
  APIs freely.
- The engine is covered by scenario tests driven through `Engine::submit`
  (combat, capture, reveal, elimination, diplomacy).
- The TUI is tested against a `FakeView` implementing `GameView`, so
  rendering logic is exercised without a terminal; input handling is tested
  through `App` methods directly.
- The repo gate is `make build`: `cargo fmt --check`, clippy with
  `-D warnings`, `cargo build`, `cargo test`.