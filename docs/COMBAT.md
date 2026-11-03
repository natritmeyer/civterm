# CivTerm — Combat

Reference for the combat model: hit points, odds, defence bonuses, siege,
city capture, and elimination. See `docs/UNITS.md` for unit stats.

## Hit points

- Combat is a race to `HIT_POINTS` (10). Each side starts at 10 HP.
- Each round, the attacker lands a hit with probability `a / (a + d)`
  where `a` and `d` are attacker/defender power.
- A side loses when its HP reaches 0. The winner survives (and is promoted
  to veteran if not already).

## Odds formula

The true win chance is `a^n / (a^n + d^n)` with `n = 10`, not the linear
share `a / (a + d)`. The two agree at even odds but diverge otherwise:
a 60-vs-90 fight wins barely 1 in 60.

`Engine::win_chance_percent(attacker, defender)` computes this exactly.

## Defence bonuses

`defender_power` stacks (multiplicatively, in this order):

| Bonus            | Multiplier |
| ---------------- | ---------- |
| Veteran          | ×3/2       |
| Mountain terrain | ×2         |
| Home city tile   | ×3/2       |
| City Walls       | ×2         |
| Fortified order  | ×3/2       |

Integer division truncates, so order matters (visible in tests).

The rival AI takes the **strongest** defender on the tile, so one fortified
unit among several puts the whole tile out of reach.

## Attacker power

- Base: `attack × 10`.
- Veteran: ×3/2.

## Siege (catapult)

- A catapult bombards a city instead of fighting its garrison.
- Spends its whole turn on one improvement, preferring City Walls.
- Rejected on a tile held by enemy soldiers with no city.
- Once the city is stripped and undefended, the catapult takes it like any
  other land unit.

## Combat resolution flow

1. `move_unit` routes sieging catapults to `bombard_city`.
2. Otherwise `select_defender` picks the strongest enemy defender.
3. Powers computed; `resolve_combat` runs the HP race.
4. Winner: defender removed (cargo disbanded), attacker advances onto the
   tile, promoted to veteran, city captured if applicable.
5. Loser: attacker removed (cargo disbanded).
6. Elimination checked for the side that lost a unit/city.

## City capture in combat

- Winning combat on a defended city tile captures the city.
- Population-1 cities are destroyed (removed, tile reverts, trade routes
  touching it purged).
- Population ≥ 2 cities transfer ownership.
- Homed units are disbanded; civilization with no cities eliminated.

## Elimination

- A civilization is eliminated when it loses its last city (with stragglers
  disbanded) or its last unit while owning no cities.
- Test fixtures with zero units are NOT eliminated.
- Eliminated players are skipped on turn advance.