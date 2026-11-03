# CivTerm — Save and Load

Reference for the save format and round-trip guarantees. See `docs/INSTRUCTIONS.md`
for player controls (`S` to save, CLI load).

## Format

- JSON (pretty-printed) in a `.civ` file.
- `SAVE_FORMAT_VERSION` = 1. Loader refuses formats newer than it understands.
- `SaveData` holds: version, turn, current player, events, RNG, `Game`,
  competition, difficulty.

## Persisted vs transient

Persisted (in `SaveData`/`Game`):
- All model state: map, units, cities, players, trade routes, difficulty.
- Engine turn, current player, events, RNG.

Transient (rebuilt empty on load via `into_loaded`):
- Rival motion record, rival wars, build completions, starvations,
  diplomat audiences, investigation, steal outcome, sabotage notice, upkeeps.

Model state round-trips through the nested `game: Game`; new fields there
need `#[serde(default)]` (or a version bump) but no capture/into_loaded change.

## Engine-level state

Adding a field to `Engine` (or to anything it owns outside `Game`) requires
updating both `SaveData::capture` and `SaveData::into_loaded` together.

## Backwards compatibility

- New fields must be optional (`#[serde(default)]`) so older files load.
- Destructive reshuffles bump `SAVE_FORMAT_VERSION`.
- Older saves (e.g. missing `trade_routes`, bare home-city ids) still load.

## Save/load API

- `save_game(path, engine, competition, difficulty)` writes JSON.
- `load_game(path)` returns `LoadedGame { engine, competition, difficulty }`.

## Validation

- Missing file → `SaveError::Io`.
- Corrupt JSON → `SaveError::Json`.
- Newer version → `SaveError::Version`.