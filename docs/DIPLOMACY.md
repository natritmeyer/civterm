# CivTerm — Diplomacy & War

Reference for diplomacy, war, city capture, and diplomat actions. See
`docs/UNITS.md` for unit stats and `docs/INSTRUCTIONS.md` for controls.

## Contact and relations

- Civilizations start unmet. Meeting a rival triggers a diplomacy dialog:
  declare war or stay at peace.
- Relations are per-pair: at war or at peace (default: neither, until met).
- `Game::declare_war` / `make_peace` update both sides.

## Peace

- At peace, units cannot enter foreign territory (peaceful-tile-occupied
  rejection).
- Units may still meet and trade (Caravan enters at peace).

## War

- War allows invasion and city capture.
- Rivals may declare war on the human (except on Easy difficulty).
- A rival declares war when a military unit stands beside a human unit or
  city whose strongest defender it outguns
  (`attacker_power > defender_power`, or `>=` on Hard), and clears the
  difficulty's minimum win chance. Easy rivals never initiate war.
- War is faction-wide: the first winnable encounter draws the whole rival in.
- A declaration is announced by a war-declaration window (drained after
  EndTurn).

## City capture

- An at-war unit walking onto an undefended foreign city tile captures it
  (if `enters_to_conquer()`).
- A defended city requires combat; winning on the city tile captures it.
- Ownership transfers immediately (city tile wears victor's colour).
- Population-1 cities are destroyed (removed, tile reverts to plain terrain).
- Population ≥ 2 cities are captured and keep producing for the new owner.
- All units homed to the fallen city are disbanded.
- A civilization with no cities is eliminated (stragglers disband).

## Diplomat entry

- A diplomat may enter a foreign city whether at peace or at war.
- Walking in does not capture the city (`enters_to_conquer()` false).
- Three places enforce this: peaceful passage cleared, routed away from
  combat (`conducts_business`), capture path checks `enters_to_conquer`.
- A diplomat entering a city records a `DiplomatAudience` and opens the
  action window (human only).

## Diplomat actions

Performed from inside a foreign city via the action window. All spend the
diplomat. Costs in gold.

| Action              | Cost (gold)              | Notes                                                 |
| ------------------- | ------------------------ | ----------------------------------------------------- |
| Investigate City    | 25                       | Buys a look; opens the city as a read-only report     |
| Steal Technology    | 50                       | Steals current research target if known, else priciest |
| Industrial Sabotage | 50                       | Destroys one improvement (prefers City Walls)         |
| Incite a Revolt     | 100                      | Buys the garrison; declares war                       |
| Subvert City        | 200 per population       | Takes the city outright; clears the tile              |

- Establish Embassy and Meet with King are deliberately absent.

### Investigate City
- Records the city on `Engine.investigation`; TUI opens it as a read-only
  report. Does not grant free later access.

### Steal Technology
- Picks the player's current research target if the rival knows it,
  otherwise the priciest researchable advance. If nothing takeable, spends
  nothing and the diplomat walks back (refusal window).

### Industrial Sabotage
- Destroys one improvement (prefers City Walls). Never empty; refused if
  the city has no improvements.

### Incite a Revolt
- Requires a garrison to bribe; empty cities refuse. The garrison defects
  to the player (re-homed to the diplomat's city); war is declared.

### Subvert City
- Takes the city outright for `200 × population` gold; clears the tile.

## Diplomat withdrawal

- Esc on the action window submits `WithdrawDiplomat`: the diplomat steps
  back to the tile he came from with his move budget restored.
- A blocked row (OK) neither acts nor closes the window.
- A refused Steal (empty cupboard) also walks the diplomat back unconsumed.

## Transport and diplomacy

- A Caravan enters a foreign city at peace (like a diplomat) to trade;
  at war the crossing is refused.

## Notes

- `Engine::diplomat_blocker` is the single source of row legality, called
  both to grey rows and before acting.
- The action window is modal: it sits ahead of the diplomacy window in
  z-order.