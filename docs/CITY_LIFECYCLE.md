# CivTerm — City Lifecycle

Reference for founding, growth, working tiles, production, starvation, and
build completion. See `docs/CITY_IMPROVEMENTS.md` for improvements and
`docs/UNITS.md` for settlers.

## Founding

- A Settler founds a city on a land tile (`v`); the settler is consumed.
- The first city of a civilization begins research (Construction).
- The city centre is always a worked tile from founding.

## Population and food

- Each citizen eats 2 food per turn (`FOOD_CONSUMPTION_PER_CITIZEN`).
- Growth threshold: `GROWTH_FOOD_PER_CITIZEN × population` food stored.
- Granary (×5/4) and Aqueduct (×3/2) boost food income.
- On growth, population increments (capped at `MAX_POPULATION` = 99).
- Worked tiles are auto-assigned on growth (one ring tile per citizen,
  preferring most productive).

## Working tiles

- Each citizen works one tile in the city's 21-tile footprint.
- Worked tiles yield food, shields (resources), and trade.
- `auto_assign_work` adds the most resource-productive available tiles.
- The city centre is always worked.

## Production

- Cities accumulate shields toward a unit or improvement target.
- On completion, the item is created: units spawn on the city tile (veteran
  if Barracks); improvements are installed.
- Rush-buy (`b` in city window) finishes the build for
  `2 × (cost − stored)` gold (min 1).

## Starvation

- A city with an empty granary and a food deficit is starving.
- Population loss (`lost_citizen`) only when population > 1; size-1 cities
  survive on nothing.
- Starving cities log "{city} is starving"; lost citizens push a
  `StarvationNotice` (human only) drained into single-OK windows after
  EndTurn.

## Build completion notice

- A city finishing a build pushes `BuildComplete` (human only).
- Windows queue one after another; OK acknowledges, "Next Order" opens that
  city's window.
- Suspended behind city/research windows; resumed on close.

## City capture and destruction

- See `docs/DIPLOMACY.md` and `docs/COMBAT.md`.
- Population-1 cities are destroyed; larger ones transfer ownership.
- Homed units are disbanded; cityless civilizations eliminated.