# CivTerm — Units

Detailed reference for every unit in the game: stats, requirements, and the
mechanics that distinguish each class. See `docs/INSTRUCTIONS.md` for the
broader rules (combat, transport, diplomacy) and `docs/TECH_TREE.md` for
the advancement prerequisites.

## Stat columns

- **Attack / Defence** — base combat power. Combat is a race to 10 hit
  points; each round the attacker lands a hit with probability
  `a / (a + d)`.
- **Moves** — movement points per turn (spent in thirds along roads).
- **Cost** — shield cost to produce.
- **Requires** — advancement needed to build (none = available from the start).

## Land units

| Unit     | Attack | Defence | Moves | Cost | Requires         | Notes                                                  |
| -------- | ------ | ------- | ----- | ---- | ---------------- | ------------------------------------------------------ |
| Settler  | 0      | 1       | 1     | 60   | —                | Founds cities; can work terrain improvements           |
| Militia  | 1      | 1       | 1     | 10   | —                |                                                        |
| Phalanx  | 1      | 2       | 1     | 15   | Bronze Working   |                                                        |
| Legion   | 3      | 1       | 1     | 15   | Iron Working     |                                                        |
| Chariot  | 4      | 1       | 1     | 15   | Wheel            |                                                        |
| Cavalry  | 2      | 1       | 3     | 20   | Horseback Riding |                                                        |
| Knight   | 4      | 2       | 3     | 20   | Chivalry         |                                                        |
| Catapult | 6      | 1       | 1     | 20   | Wheel            | Siege engine: bombards city walls, never fights units   |
| Diplomat | 0      | 0       | 2     | 20   | Writing          | Enters foreign cities; special actions                  |
| Caravan  | 0      | 1       | 1     | 30   | Trade            | Creates trade routes on entering a city                |

## Naval units

| Unit    | Attack | Defence | Moves | Cost | Requires    | Notes                                      |
| ------- | ------ | ------- | ----- | ---- | ----------- | ------------------------------------------ |
| Trireme | 1      | 0       | 3     | 30   | Bronze Working | Carries 2 land units; coastal build only |
| Sail    | 1      | 1       | 5     | 20   | Map Making  | Carries 2 land units; coastal build only   |
| Frigate | 2      | 2       | 7     | 40   | Magnetism   | Carries 2 land units; coastal build only   |

## Special abilities

### Settler
- Founds a city on a land tile (`v` key). The settler is consumed; the
  city is named after the civilization's capital.
- Can work terrain improvements (Irrigation, Mine, Road) via the command
  picker (`w`).
- Only settlers may found cities.

### Catapult (siege engine)
- `attacks_units()` is false and `sieges()` is true: it never strikes a
  garrison, only bombards a city.
- From where it stands, it bombards one improvement per turn (preferring
  City Walls), spending its whole turn.
- It may enter a city tile to bombard, but is refused on a tile held by
  enemy soldiers with no city.
- Once the city is stripped of improvements and undefended, it takes the
  city like any other land unit.

### Diplomat
- `enters_to_conquer()` is false: walking into an undefended enemy city does
  not capture it.
- May enter a foreign city whether at peace or at war (no casus belli
  needed). Never routed to combat against the garrison.
- 2 movement points (all other land units are 1 or 3).
- On entering a rival city, opens the diplomat action window: Investigate
  City, Steal Technology, Industrial Sabotage, Incite a Revolt, Subvert City.
- Walking in at war onto a tile with enemy units but no city still resolves
  as combat (at 0 attack he loses and is deleted) — intended behavior.

### Caravan
- `trades()` is true: may enter a foreign city at peace (like a diplomat),
  but only in peace — at war the crossing is refused.
- Still `enters_to_conquer()`, so at war on an undefended city it captures
  like any other unit.
- On entering any city, attempts to establish a lasting trade route; on
  success the caravan is consumed. Refused if the destination market is full
  (`MAX_TRADE_ROUTES` = 3 endings per city).

### Naval transports
- Trireme, Sail and Frigate carry up to 2 land units (`carry_capacity()`).
- Boarding: a land unit moves onto a friendly transport's tile (free, no
  movement cost).
- Disembarking: a transported unit moves onto an adjacent land tile (free).
- Transported units are inert: no movement except ashore, no fortify, work,
  sentry, or founding. Cargo never fights.
- Ships can only be built in coastal cities (`borders_water`).

## Unit orders

| Order      | Effect                                                                   |
| ---------- | ------------------------------------------------------------------------ |
| Idle       | Default; unit can move, fight, or take another order                      |
| Fortify    | ×3/2 defence; spends the turn; unit leaves the command loop until moved   |
| Sentry     | Removes the unit from the Tab cycle until unsentried                      |
| Improving  | Settler spends turns building Irrigation (2), Mine (3), or Road (2)       |

- `f` fortifies (land units only; ships cannot dig in at sea).
- `s` stands sentry.
- `c` cancels the current order.
- `w` opens the command picker (fortify / sentry / work / disband / cancel).
- `Unfortify` / `Unsentry` restore the unit to the command loop without
  spending a turn.
- A fortified unit on a city tile is hidden on the map (stowed as garrison).

## Defence bonuses (stack multiplicatively)

Collected in `defender_power`:

| Bonus            | Multiplier |
| ---------------- | ---------- |
| Veteran          | ×3/2       |
| Mountain terrain | ×2         |
| Home city tile   | ×3/2       |
| City Walls       | ×2         |
| Fortified order  | ×3/2       |

A fortified veteran behind walls on its home tile is 3× its bare defence.

## Veteran status

- Units produced in a city with a Barracks muster as veterans.
- A unit that wins a fight is promoted to veteran.
- Veteran gives ×3/2 attack and defence.

## Movement and terrain

- Terrain movement costs apply; mountains cost 3, forests/jungle/swamp 2,
  others 1 (in thirds-of-a-point units internally).
- Roads: a road-to-road step costs 1/3 of a movement point, letting a unit
  travel three times as far along a road. Stepping onto/off a road pays the
  full terrain cost.
- Ships move on water only; land units on land only. Boarding is the only
  lawful land↔water transition.

## Combat outcomes

- A city falls to the attacker the moment its guard breaks (undefended city
  walked onto, or combat won on a defended city tile). Ownership transfers
  immediately.
- A conquered city of population 1 is destroyed (removed, tile reverts to
  plain terrain); population ≥ 2 is captured.
- Conquered city's homed units are disbanded; a civilization with no cities
  is eliminated.
- A unit fortifying on a city tile becomes the garrison and is hidden.

## Transport invariants

- Max 2 land units per transport.
- Transported units have no map square of their own (`units_at` omits them;
  `player_units` keeps them for selection).
- When a carrier is removed, its cargo goes with it.

## Coastal shipbuilding

- Ocean-going units (Trireme, Sail, Frigate) can only be built in cities
  that border water (`borders_water`: at least one of 8 neighbours is ocean).
- Enforced in both the production list and `set_production`.

## Notes

- `UnitClass::Diplomat.moves()` is 2; all other land units are 1 or 3.
- Ships cannot fortify (`can_fortify()` false) but may stand sentry.
- A unit's home city is `Option<CityId>`; a starting settler has `None`.