# CivTerm — City Improvements

Detailed reference for every city improvement: cost, upkeep, prerequisites,
and the engine effect each one actually has. See `docs/INSTRUCTIONS.md` for
the player-facing summary and `docs/TECH_TREE.md` for the advancement tree.

## Overview

- Improvements are built in cities like units, accumulating shields each
  turn until the cost is reached.
- Each improvement costs gold per turn to keep (`upkeep`); unpaid upkeep can
  disband a unit.
- A city may build each improvement at most once.
- Some improvements require an advancement; Barracks requires none.

## Improvements table

| Improvement | Cost (shields) | Upkeep (gold) | Requires          | Effect                                              |
| ----------- | -------------- | ------------- | ----------------- | --------------------------------------------------- |
| Barracks    | 15             | 1             | —                 | New units produced as veterans                      |
| City Walls  | 15             | 1             | Masonry           | ×2 defence for all defenders in the city            |
| Aqueduct    | 15             | 2             | Construction      | Food income ×3/2                                    |
| Temple      | 20             | 1             | Ceremonial Burial | +1 happy citizen                                    |
| Library     | 20             | 1             | Writing           | Research output ×5/4 (stacks with University)       |
| Marketplace | 20             | 1             | Currency          | Gold income ×3/2 (stacks with Bank)                 |
| Granary     | 20             | 1             | Pottery           | Food income ×5/4                                    |
| Courthouse  | 20             | 1             | Code of Laws      | Not yet wired (planned: reduces corruption)         |
| Colosseum   | 20             | 4             | Construction      | +3 happy citizens                                   |
| Cathedral   | 20             | 3             | Religion          | +4 happy citizens                                   |
| Bank        | 20             | 3             | Banking           | Gold income ×3/2 on top of Marketplace              |
| University  | 20             | 3             | University        | Research output ×3/2 (stacks with Library)          |
| Palace      | 30             | 0             | Masonry           | Not yet wired (planned: capital / corruption anchor) |

## Happiness

- Every citizen starts content.
- Temple (+1), Colosseum (+3), Cathedral (+4) lift citizens into the happy
  tier, stacking but never exceeding population.
- A happy citizen is twice as productive (shields only); food, trade,
  research and gold are untouched.
- `City::happy_citizens` / `City::content_citizens` are derived; nothing
  stores happiness.

## Economy effects

### Food income
- `City::food_income(raw)` applies Granary (×5/4) and Aqueduct (×3/2)
  multipliers to the raw worked-tile food yield.

### Gold income
- `City::gold_income(raw)` applies Marketplace (×3/2) and Bank (×3/2)
  to the raw tax base (worked-tile trade arrows + special-resource gold).

### Research
- `City::research()` applies Library (×5/4) and University (×3/2) to the
  diminishing-returns research output.

### Shields (production)
- `City::resource_income(raw)` scales raw shields by
  `(content + 2 × happy) / population` (happy-citizen bonus).

## Defence

- City Walls give ×2 defence to all defenders in the city
  (`CITY_WALLS_DEFENSE_BONUS`), and are the preferred target of catapult
  bombardment.

## Veterans

- A city with a Barracks trains produced units as veterans
  (`Unit::promote()` on spawn).

## Upkeep

- `City::upkeep()` sums each improvement's maintenance.
- `Engine::charge_upkeep` charges the owner once per round; a shortfall
  clamps the treasury to zero and disbands the costliest unit.

## Unwired improvements

- **Courthouse** — intended to reduce corruption (Civ 1: −50%). No
  corruption system exists yet; planned as part of the Governments feature.
- **Palace** — intended as the capital and corruption anchor (distance from
  the Palace). No capital logic exists yet; planned alongside Governments.

See `scratch/backlog.md` ("Unwired improvements") for the planned wiring.

## Notes

- Improvements are stored as `Vec<CityImprovement>` on `City`; duplicates
  are prevented by the production picker (`production_choices` filters out
  held improvements).
- `CityImprovement::required_advancement()` gates availability via the
  player's discovered advances.