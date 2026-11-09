# CivTerm — Rival AI & Turn Resolution

Reference for how rivals act during `EndTurn`. See `docs/DIFFICULTY.md` and
`docs/UNITS.md`.

## Turn resolution

- `Command::EndTurn` resolves the whole round on the engine: each rival
  civilization takes its turn, then control returns to the human and the
  turn number increments once.
- Rivals act in player order; eliminated players are skipped.
- The human's turn begins fresh (units' moves restored).
- Rival moves are recorded as `RivalMotion` and replayed by the TUI (600ms
  per step; Space/Enter/Esc skips the replay).

## Rival turn (`run_rival_turn`)

Passes, in order:

1. Research — pick/advance research.
2. Production — set production targets.
3. Settle — move settlers to found cities.
4. Garrison — fortify defenders in cities.
5. Warfare — move/attack, declare war.
6. Rush-buy — if at war, rush builds while gold allows (reserve 10).

## Research / production

- Deterministic: never consults `self.rng`.
- Production chosen from `production_choices` (inherits coastal rule).

## Settlement (`best_settlement_site`)

Tiers measured against the union of all civilizations' 21-tile working
grids:

1. Open — shares at most 3 tiles (≥ Chebyshev 4 off every city).
2. Acceptable — shares at most 6 tiles (≥ Chebyshev 3).
3. Frontier — march to nearest explored land tile bordering unexplored ground.
4. Crowded — best-scored tile (only on a fully explored map).

Within a tier: best score, then nearest to settler, then smallest tile.
A settler never enters the crowd unless no alternative; sites are only
chosen from explored tiles.

## Movement

- Every landing records a `RivalMotion { unit, from, to, battle }`
  through `record_rival_step`, gated `owner != human`.
- Recorded before combat resolves (repelled attackers still need replay).
- Sieges record no motion (no tile change); final capture does.

## Warfare

- A rival fights only fights it expects to win.
- Declares war on the human when a military unit stands beside (8
  neighbours, wrap) a human unit or city whose strongest defender its
  power outguns (`attacker_power > defender_power`, or `>=` on Hard), unless
  Easy (never declares war).
- Never attacks below the difficulty's minimum win chance.
- War is faction-wide.
- A siege engine never declares war to bombard; it sieges only an already-
  declared war.
- Declarations recorded as transient `RivalWar { rival }`, drained and
  announced by the TUI.

## Rush-buy

- Runs at end of `rival_units`, only when at war.
- Walks cities in id order; rushes each current build while cost fits
  beneath `RIVAL_GOLD_RESERVE` (10).

## Difficulty-gated behavior

- **Easy**: militia spam (original behavior); no improvements, no terrain
  development, no naval expansion.
- Militia production is capped at 7 per city.
- **Normal (medium)**: rivals build city improvements (cheapest available)
  once their city count reaches the expansion target, and idle settlers
  develop terrain (irrigation, mines, roads) around owned cities instead of
  idling.
- **Hard**: additionally builds ocean-going units in coastal cities and
  transports settlers/military across water to other continents; keeps the
  Normal improvements and terrain development.

## Determinism

- Decisions never use `self.rng`; only combat resolution advances the RNG.