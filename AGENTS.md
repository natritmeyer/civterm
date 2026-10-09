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

Run `make build` after any change. Current test baseline: 795 passing unit
tests. Keep this baseline line and the README's badge (`tests-N%20passing`)
in step with the actual count whenever tests are added or removed.

## Git policy

Never commit code — staging, committing, and pushing are the human's job.
Leave changes in the working tree; the human decides when and how to commit.

## Architecture

Top-level modules (`src/lib.rs`): `crash_log`, `game_engine`, `model`, `tui`,
`utils`.
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
      `diplomat_actions.rs`, `turns.rs` — private modules, one `impl Engine`
      block each by concern (`combat.rs` owns `HIT_POINTS`).
    - `diplomat_actions.rs` — `DiplomatAction`/`DiplomatOption`/
      `DiplomatAudience`/`DiplomatError` (re-exported as
      `game_engine::DiplomatAction` and friends) plus the action window's
      engine half: `diplomat_options`, `record_diplomat_audience`,
      `drain_diplomat_audiences`, `perform_diplomat_action` and
      `withdraw_diplomat`.
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
- Conquest is a property of the unit, not of the tile: `UnitClass::
  enters_to_conquer` is false for the Diplomat alone. A diplomat walks into an
  undefended enemy city and does **not** take it, because taking a city by
  diplomacy is Incite a Revolt or Subvert, both of which cost gold. Every other
  unit — settlers included — takes the ground it walks on.

## Diplomat entry

- A diplomat may enter a foreign city whether the two civilizations are at
  peace or at war. Walking in unannounced is what he is for; he needs no
  casus belli and so never has cause to demand one. Civ 1 lets him do this
  past any garrison, and the repo has no Zone of Control rule to lift, so this
  is the whole of the mechanic.
- That single permission touches three places, and all three are needed or the
  rule lies in one of its halves:
  - `ensure_peaceful_passage` clears the foreign-occupant flag for a
    diplomat bound for a foreign city, so the city does not bar him at peace.
  - `move_unit` sets `conducts_business` for the same pair, so a diplomat is
    never routed to `resolve_combat` — at 0 attack he would lose every fight
    to the garrison waiting inside, and be deleted on arrival for the crime of
    arriving.
  - the capture path checks `enters_to_conquer`, so walking in does not take
    the city.
- Nobody else gets any of this: a warrior is still held out of a peaceful
  neighbor's city, and a settler still takes an undefended enemy one. The
  exception is the unit's alone, which is why it is data on `UnitClass` and
  not a special case spelled out per call site.
- A diplomat walking into a city is an ordinary move, so it spends movement,
  reveals, and records a `RivalMotion` like any other step. What he may *do*
  once he is inside is the action window below.
- `UnitClass::Diplomat.moves()` is 2, where every other land unit is 1 or 3.
  He is built for walking into places, so reaching the first one should not use
  him up, and the step left in hand is what lets a player who dismisses the
  window walk back out again.
- One case is still unresolved and should not be quietly folded in here: a
  diplomat at war stepping onto a tile held by enemy units with **no city** on
  it still resolves as combat, and at 0 attack he loses it and is deleted. Civ
  1's answer belongs with the city-capture ransom, so fix it there rather than
  widening entry here.

## The diplomat action window

- Walking the human's diplomat into a foreign city records a `DiplomatAudience`
  from the plain-move tail (human only, like every other rival-motion filter)
  and the TUI drains it into one modal window. The record is transient; the one
  thing the diplomat *learns* (`City::investigated`) is model state and does
  survive a save.
- Five actions, in this fixed order and no others — `DiplomatAction::ALL` is the
  single source of that order, and both the window and `diplomat_options` read
  it: Investigate City (25), Steal Technology (50), Industrial Sabotage (50),
  Incite a Revolt (100), Subvert City (200 **per head of population** — the
  city the diplomat stands in is handed to `DiplomatAction::cost`, so a city
  of three costs 600). Establish Embassy and Meet with King are deliberately
  absent. Every one of them spends the diplomat.
- `Engine::diplomat_blocker` is where a row's legality lives, and it is called
  twice for the same action: once by `diplomat_options` to grey the row out and
  supply the sentence the window prints under the list, and again by
  `perform_diplomat_action` before anything happens. The window snapshots the
  options when the diplomat arrives, so the second check is not redundant — gold
  can run out between the window opening and the player pressing OK.
- A blocked row is not a choice: OK on one neither acts nor closes the window.
  Esc always closes — and closing is not just dismissal, it is an undo: the
  window's state carries `from` and `moves_before` (the tile the walk-in left
  and the budget it carried), and `close_diplomat_actions` submits
  `Command::WithdrawDiplomat`, so the diplomat steps back onto `from` with his
  move budget restored. A player who turns down every offer is handed his
  diplomat back outside the city, unconsumed, rather than stranded inside one.
- Investigate City buys a look, and every look is a fresh purchase: the engine
  records the city on the transient `Engine.investigation` (pushed by
  `perform_diplomat_action`, drained by `drain_investigation`, rebuilt empty by
  `into_loaded`), and the TUI opens that rival city's window there and then, as
  a read-only report over the map. `investigated` holds no veto on later
  investigations — a city can be probed again and again, at 25 gold and a
  diplomat a look. What it *does* keep is the one free door shut: the report is
  a showing, not a door, so a foreign city stays unselectable on the map and
  the intel is only ever rebought for gold, never tapped for free later. The
  window renders a foreign city only once `investigated` is set, hides the
  Change and Unfortify affordances (the player edits no one else's order and
  releases no one else's garrison), and the Change click is additionally
  guarded by ownership so the production picker can never open on a rival city.
- Incite and Subvert need nothing but the gold and whatever else each is
  aimed at: the diplomat acts alone, and no unit of his own side is ever
  required. Incite buys the rival garrison standing in the city and declares
  the war — with no garrison there is nothing to bribe, so an empty city
  refuses it. Subvert asks nothing of the garrison at all: gold alone is the
  price, at `SUBVERT_CITY_COST` per head of population, and it takes the city
  outright and clears the tile, which is what distinguishes them.
- The garrison a diplomat buys is the player's unit from the moment the
  action lands. `Unit::defect_to` re-homes it to the diplomat's city, and the
  incite arm cancels the rival's fortify order and restores the spent budget
  — a real garrison is both of those things, and either one alone leaves the
  man stuck outside the ordinary command cycle (Tab skips a Fortified unit
  with no moves). The city he was bought out of is still the enemy's, so his
  first duty is the walk-out: `standing_in_foreign_city` makes fortify,
  sentry and work refuse on the rival square, and `available_commands`
  offers the picker nothing there. Cancelling a release (Unfortify,
  Unsentry, CancelOrder) stays allowed, because undoing an order is not
  taking one, and the walk-off needs no permission it does not already have.
- Steal Technology announces itself, and the pick is
  not luck but priority: the player's current research target if the rival
  knows it, otherwise the priciest advance the player could begin researching
  next (`researchable_advancements`). An advance the player cannot yet work on
  is no prize, so a rival whose cupboard holds nothing researchable is robbed
  of nothing. The engine records what came away — `StealOutcome::Stolen`
  (which advance, whose it was) — or the refusal `StealOutcome::NothingToSteal`
  (both transient, drained by `drain_steal_outcome`) and the TUI opens a
  single-OK window of its own that closes on acknowledgement like the
  war-declaration window and is gone from the draw and the key and mouse guards
  the moment it is. The theft itself is already spent and logged as the event
  "Unit N steals ..."; the window is the receipt.
- An empty cupboard is a refusal, not a dead row: Steal Technology is always
  actionable (there is no `NothingToSteal` blocker on it), and the attempt
  against a rival with nothing takeable spends nothing. The refusal's window
  says so, and `diplomat_actions_confirm` then submits the same
  `Command::WithdrawDiplomat` a dismissal would, so the diplomat walks back
  onto the tile he came in from with his budget restored — an attempt that
  takes nothing hands the man back unconsumed, exactly like the Esc that gave
  up on the whole window.
- Industrial Sabotage also announces itself: the engine records `SabotageNotice`
  (the improvement that came down and the city it stood in, transient, drained
  by `drain_sabotage_notice`) and the TUI opens a single-OK window naming both,
  in the same shape and z-order as the theft's window. It differs from Steal
  Technology in one place only: it is never empty — `diplomat_blocker` still
  refuses it when the city has no improvements to break
  (`DiplomatError::NoImprovementsToDestroy`) — so its window always reports
  damage and its diplomat is always spent. The damage is already logged as the
  event "Unit N sabotages ..."; the window is the receipt, read from the same
  `diplomat_actions_confirm` drain as the theft.
- The window is modal like the others — `window_is_open`, `modal_open`,
  `hover_blocked`, a key guard ahead of the diplomacy guard and a mouse guard
  ahead of the diplomacy guard. It sits ahead of the diplomacy window because
  first contact with the city's owner can be recorded on the very same step,
  and the more specific window belongs on top.
- The diplomat guard returns `false` — it consumes the key and says nothing to
  the run loop, exactly like every modal except the quit dialog. `handle_key`'s
  flag means *quit the process*, and `true` from any other window reads as a
  crash: the game closes silently on the first key it sees, and the exit looks
  clean because nothing went wrong. The regression test
  `a_key_in_the_diplomat_window_leaves_the_game_running` reads the flag, which
  the other tests routinely throw away.

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
- A siege engine is the exception to the rule below, and it is a policy
  exception rather than a weaker one: a bombardment never meets the garrison,
  so the power comparison does not apply to one at all, and `rival_best_attack`
  returns it a target it could never outfight. It does **not** declare a war to
  do so — a bombardment strips walls but takes nothing — so `rival_warfare`
  skips a siege engine entirely while at peace and sieges only a war already
  declared. See `Siege` for the mechanics.
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

## Defence

- `defender_power` is where every defensive bonus is collected, so the
  multipliers stack multiplicatively and in one readable place: veteran
  (×3/2), mountain terrain (×2), the unit's own home city
  (×3/2), city walls on that tile (`CITY_WALLS_DEFENSE_BONUS`, ×2), and the
  fortified order (`FORTIFIED_DEFENSE_BONUS`, ×3/2). A fortified veteran behind
  walls on its home tile is therefore three times its bare defence.
- The fortified bonus belongs to the **order**, not to the unit or its move
  budget: a unit that fortified and then spent its moves is still a wall, and
  `cancel_order` hands the bonus straight back. Nothing else pays out — a
  sentry or idle unit defends at its ordinary value, so the player is choosing
  `f` and not merely occupying the tile.
- Every multiplier lives as a named `const` in `combat.rs`, because these are
  tuning numbers and a bare `* 3 / 2` in the middle of the function hides
  them. Note the integer division: `power * 3 / 2` truncates, so the order of
  the bonuses is visible in the tests that pin the stacked values.
- The rival AI reads `defender_power` for its own winnable-fight test
  (`attacker_power > defender_power`, plus a minimum win chance), so a
  fortified garrison raises the bar a rival has to clear without any AI change.
  The AI takes the **strongest** defender on the tile, so one fortified unit
  among several is enough to put the whole tile out of reach.

## Combat odds are a race, not a ratio

- Combat is a race to `HIT_POINTS` (10), not a single roll: each round the
  attacker lands one hit with probability `a / (a + d)`. The odds of winning
  are therefore `a^n / (a^n + d^n)`, and `Engine::win_chance_percent` in
  `combat.rs` models exactly that. It lives in `combat.rs` because it is a
  property of the fight `resolve_combat` runs, and it reads `HIT_POINTS`
  rather than repeating the number.
- The linear share `a / (a + d)` is **not** the odds. The two agree at even
  stakes — which is exactly what hid the bug — and diverge violently past
  it: 60 against 90 is two thirds of the power and wins one fight in sixty,
  not forty in a hundred. Anything reasoning about combat odds must go
  through `win_chance_percent`.
- The function lives with the fight, but the rival's *policy* stays in
  `rival_player_engine.rs`: `attacker_power > defender_power` plus
  `RIVAL_MINIMUM_WIN_CHANCE`. Note that the power comparison is the binding
  constraint in practice, since it already refuses every fight at or under
  even odds and the chance floor sits well beneath that.

## Siege

- The Catapult is a siege engine, not a warrior. `UnitClass::attacks_units`
  is false for it alone and `UnitClass::sieges` true for it alone, so the
  distinction is data on the unit rather than a special case spelled out at
  every call site. `is_military` is unaffected — a catapult is land military
  and garrisons and war AI still see it.
- A siege engine bombards a city from where it stands. It never walks onto
  the tile it is attacking, and it only takes the city once there is
  nothing left to break, so `move_unit` routes it to the combat tail
  whenever the destination holds an enemy city **with improvements still
  standing** — otherwise an undefended but walled city would fall to a
  catapult without a shot being fired. With the city stripped and no
  soldiers on it, the engine falls through to the ordinary capture path
  and takes it like any other land unit: the siege is the means, not the
  end. It cannot fight its way in, but it need not once there is nothing
  left to defend.
- A bombardment records no `RivalMotion`: the engine never changes tile, so
  there is nothing to replay, and unlike a repelled attacker it survives to
  bombard again next turn. The final capture *is* a tile change and is
  recorded like any other advance.
- `bombard_city` spends the whole turn on one improvement and prefers
  `CityWalls`, because the walls are the only improvement that is a defence
  in its own right. Tearing them down strips
  `CITY_WALLS_DEFENSE_BONUS` from every defender in the city, which is the
  entire point: a fortified walled garrison is otherwise close to
  unattackable, since a losing attacker is removed outright and no wound
  state survives between fights, so a defended city cannot be ground down by
  attrition.
- A tile held by enemy soldiers with no city on it rejects the catapult
  outright (`MoveError::CannotAttackUnits`) — it has no quarrel with a
  garrison and nowhere to bombard from.

## Coastal shipbuilding

- A city may only build an ocean-going unit (`UnitClass::can_travel_water`:
  trireme, sail, frigate) if it stands on the coast — `Game::borders_water`
  is true when any of the city tile's eight neighbours is ocean, wrapping
  east/west exactly as movement does. A ship built inland would have no tile
  to step onto and sit stranded on land forever, so the city is offered none.
- The rule is enforced in both the list and the command, the same two-place
  discipline as `diplomat_blocker`. `GameView::production_choices` filters the
  naval classes out for an inland city, so the production picker never shows a
  ship it cannot launch; `Engine::set_production` re-checks and rejects with
  `"Cannot produce …: the city is not on the coast"` (event-only, like every
  other production rejection), so a stale picker, a save, or a direct command
  cannot slip a ship past the list.
- Because the rival AI picks its production from `production_choices`, it
  inherits the rule for free: a rival inland city builds no navy. No separate
  AI check is needed, and none is written.
- A ship cannot take the `f` fortify order: `UnitClass::can_fortify` is false
  for `can_travel_water` classes, since there is no ground at sea to dig into.
  A ship may still stand sentry. The rule is enforced in the same two places —
  `command_picker::available_commands` omits the Fortify row for a ship, and
  `Engine::fortify` re-checks and rejects with `"Unit N cannot fortify at
  sea"` (event-only), spending nothing — so the direct `f` key cannot slip a
  fortify past the picker. No AI change is needed: `is_military` already
  excludes naval units, so `rival_garrison` never tries to fortify one.

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

## Event log ownership

- An event says whose story it is. `Event::new` builds one about nobody in
  particular — a rule rejection the player caused, which stays visible. The
  constructors that matter are `Event::for_player` (a civilization's own
  business: its units moving, its city growing, its research) and
  `Event::between` (a clash between two, told to both).
- `record_events` keeps only `is_about(PlayerId::new(0))`, so a rival's turn
  start, its settlers walking and its cities producing never reach the window
  the player reads. Filter at the TUI, not the engine: the engine has no idea
  which civilization is at the keyboard.
- A battle is `between`, never `for_player`. A rival attacking the player is
  the player's news even though the aggressor belongs to someone else, and
  `is_about` matches either side of a `Both`.
- Attribution is data, not a naming convention, because the messages cannot be
  filtered by text: `"Unit 1 moves SW"` carries no civilization name at all.
- `Event.about` is `#[serde(default)]`, so events written by older builds —
  and `SaveData` does persist `events` — still load as `Everyone` rather than
  silently vanishing from the log.

## The city window is modal

- `selected_city` is the city window's state, and while it is `Some` the map is
  inert. `handle_playing_key` returns early for every key except Esc, so no
  map command reaches the tile the panel covers: a stray space or Enter cannot
  end the turn (they are the end-turn key), and the movement keys, `Tab`, `v`,
  `w`, `f` and `c` cannot act behind it. Esc closes the window.
- That guard sits **after** the production picker's, so the picker floating over
  the window keeps its own keys, and after the research/diplomacy/build-notice
  guards, which open over the map rather than from it.
- Mouse presses follow the same rule. A press inside the window acts on its
  buttons; a press anywhere outside dismisses the window and is consumed, so it
  never becomes a drag or a `map_click` on the tile underneath. Clicking out is
  the dismissal, so the window needs no key of its own beyond Esc.
- `selected_city.is_some()` counts as modal in `modal_open`, so a pending
  auto-advance cannot fire under the window and move the selection out from
  under the player.
- The rule is one-directional on purpose: dismissing the window is always
  allowed, so no input can leave the player stuck behind a panel.

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
  the next (closing the window once the queue empties); `Next Order` pops it
  too and opens *that* city's window instead.
- `Next Order` sets `BuildNoticeState::suspended`, which parks the queue: the
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
  dropping, for the same reason `Next Order` parks rather than drops.
- The notice is modal while it shows, exactly like the war-declaration window:
  `q` does not reach the quit path until the queue is answered.

## City starvation

- A city with an empty granary and a food deficit is starving. The model
  reports that raw condition on `CityTick.starving` — a size-one city needs the
  warning too — but the population loss is a separate flag, `lost_citizen`,
  which is only set when the city actually shrinks. The last citizen is never
  taken, so a size-one city survives on nothing and `lost_citizen` stays false.
  The window reports a loss from `lost_citizen`, never from `starving`: a
  "lost 10,000 population" message must never be told about a city that kept
  everyone.
- `City::tick` does the shrinking itself when `lost_citizen`; the engine only
  records and announces. `process_cities` logs the event
  `"{city_name} is starving"` whenever the city is starving (human or rival,
  filtered at the TUI like every event), and pushes a `StarvationNotice` onto
  the transient `Engine.starvations` when a citizen was lost — human only,
  like every rival-motion filter, because a rival's hungry city is news, not a
  decision for the player.
- The TUI drains the record (`drain_starvations`, oldest first) on EndTurn into
  a loop of single-OK windows, one per hungry city, in the same shape as the
  war-declaration and build-completion loops. OK acknowledges the loss and
  shows the next; the window closes once the queue empties. It is modal while
  it shows — `window_is_open`, `modal_open`, `hover_blocked`, a key guard ahead
  of the build-completion guard and a mouse guard ahead of the diplomacy guard
  — and both the queue and its rectangle are cleared by `reset_setup` and
  `check_game_over`.
- `StarvationNotice` is transient: `Engine.starvations` is rebuilt empty by
  `Engine::into_loaded` and never pushed into `SaveData`.

## Save and load impact

New features are judged for their save impact before they are built:

- Ask first what state a feature introduces and whether it must survive a
  save/load round trip. Transient state — e.g. `Engine.motion` (the rival
  replay record), `Engine.builds` (the build-completion queue),
  `Engine.investigation` (the report window the next investigation opens),
  `Engine.steal` and `Engine.sabotage` (the theft and sabotage report windows),
  the battle flash, UI-only selection — is rebuilt fresh (`Engine::into_loaded`
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
  as the city's garrison, so the tile keeps showing its population (and
  the city's name label) as if unoccupied. Any other unit still paints its
  letter ahead of the population.
- Transient overlays (the battle flash, the rival-move replay) are painted in
  a final pass after tiles, markers and city-name labels, so they sit at the
  top of the z-order over everything the map draws. The flash goes last of all,
  over the replayed rival glyph too, so an arriving attacker is eclipsed by the
  💥 it triggered.
- A city tile shows its population, and `MAX_POPULATION` (99) caps it: the
  number the map must be able to draw is what bounds the rule, not the other
  way round. `population_text` hands back one digit for a small city and two
  from ten upwards, and the tile's spare column is the second digit's home, so
  a two-digit city takes the improvement watermark's place for itself (a small
  city still keeps it). The left column still holds exactly one character:
  ratatui's `set_symbol` stores its argument verbatim, so handing it the whole
  two-digit size would paint both digits into one cell and let the renderer
  spill them east. The cap doubles as a layout guard, since a save written
  before cities were capped can carry a bigger number and a third digit would
  land in the neighbouring tile.
- A window's rectangle is recomputed on **every** frame it is up, and recomputed
  again by the mouse guard from whatever the last frame recorded, so the
  geometry of every floating window must be **total over every `Rect` a terminal
  can report**. `App::draw` asks for the rect before the widget gets a chance
  to bail on a too-small area, so a plain `area.width - width` in a centering
  helper overflows and takes the game down mid-decision the moment a terminal is
  dragged down to a column or two. Every window therefore takes its `centered`,
  `inner`, `button_row_y` and `draw_border` from `tui/window_geometry.rs`, which
  saturates and clamps inside the area; a window with its own arithmetic (a
  button's `right() - 17`, the city window's three-column bottom band) has to
  saturate the same way. `window_geometry`'s `every_window` test hit-tests all
  ten windows at twenty terminal sizes, which is what keeps one window's copy
  from reintroducing the overflow the module was written to remove.
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

## Crash reporting

- `crash_log` is installed from `main` **before** the terminal is touched, so a
  crash from the moment raw mode goes on is reported *and* the terminal is put
  back. Restoring matters as much as logging: a panic that leaves raw mode on
  with the alternate screen up takes the player's shell with it, which is
  exactly how a crash reports nothing at all.
- `restore_terminal` is idempotent and gated on `terminal_active`, which `main`
  sets once the alternate screen is up. A crash *before* that — during install,
  during map generation — must not emit escape sequences at a terminal that was
  never taken over, so the gate is what lets the panic hook, the signal handler
  and the normal exit path all share the one teardown instead of three copies
  that drift.
- Nothing in the module may panic: a crash reporter that crashes the game at
  startup has defeated itself. `install_signal_handler` wraps a call that is
  documented to `assert!`, `recent_notes` takes the ring without blocking (the
  thread being crashed is likely the thread holding it), and every I/O result is
  discarded. Reports append to `civterm-crash.log`, or wherever
  `CIVTERM_CRASH_LOG` points.
- Breadcrumbs belong where the crash localises. `App::run` records every key and
  mouse event and a note on each phase change; `Engine::submit` records the
  command, and it records it *inside* the engine rather than at each of its
  twenty-odd call sites, so a command issued by a rival's turn or by a pending
  auto-advance is in the ring too. The ring holds 40 notes and flattens each to
  one line, so a note cannot push the report out of shape.
- SIGSEGV, SIGILL and SIGFPE are deliberately **not** handled. signal-hook
  refuses to register a hardware fault through its checked API — `Signals::new`
  asserts rather than returning an error, which took the whole process down the
  first time this was written — and getting past that means `unsafe` in a crate
  that has none, for a fault only a dependency could raise. SIGABRT is handled,
  and that is the one that matters: a double panic becomes an abort and prints
  nothing a panic hook could have caught first.

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
