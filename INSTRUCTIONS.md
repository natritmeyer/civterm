# CivTerm — Player Instructions

## Overview

CivTerm is a terminal Civilization-style game. You lead a civilization from a single settler to a sprawling empire: found and grow cities, research advances, build units and improvements, explore the world, and deal with rivals by diplomacy or war. The human player is always civilization 0 (English by default).

## Starting a game

1. Launch `civterm`, or `civterm path/to/save.civ` to load a saved game directly.
2. Choose your **civilization**, **competition** (number of rival civilizations, 1–7) and **difficulty** (Easy / Normal / Hard).
3. The map is generated with terrain, resources, and rival starting positions.
4. Your first unit (a Settler) is selected automatically.

## Difficulty levels

| Level  | Starting gold | Rivals declare war? | Rivals attack at even odds? | Rival win-chance floor | Rival economy                                                    |
| ------ | ------------- | ------------------- | --------------------------- | ---------------------- | ---------------------------------------------------------------- |
| Easy   | 50 gold       | No                  | No                          | 10%                    | 3/4 production, 3/4 research, slower growth (3/2 food), 3/4 gold |
| Normal | 0             | Yes                 | No                          | 10%                    | Identity (no scaling)                                            |
| Hard   | 0             | Yes                 | Yes                         | 0%                     | 5/4 production, 5/4 research, faster growth (3/4 food), 5/4 gold |

Only rivals are scaled; your economy is never affected. On Easy the neighbors stay peaceful unless you declare war.

## Main screen layout

- **Left column**: info panel (selected tile, units, city details) and event log.
- **Map**: two-character-wide tiles; population shown on city tiles.
- **Bottom help bar**: toggled with `?`; lists the keys available in the current context.
- **Event log**: toggled with `e`; shows the last few messages (your civilization's news only).

## Controls

### Movement & selection

- Arrow / `h j k l` style keys and diagonals (`y i n ,`) move the selected unit.
- `Tab` — jump to the next unit that still has movement.
- `j` — centre camera on the selected unit.
- Mouse drag pans the map; clicking a nearby tile moves the selected unit.

### Unit orders

- `f` — fortify (land units only).
- `s` — stand sentry.
- `c` — cancel the current order.
- `w` — open the command picker (fortify/sentry/work/disband etc.).
- `v` — found a city (Settler).
- `d` — disband the selected unit (ships with cargo confirm first).
- `Space`/`Enter` — end turn.

### City window

- Click a city tile to open its window.
- `b` — rush-buy the current production.
- `Esc` — close.
- Mouse: **Change** opens the production picker; **Buy** rushes; **✕** closes.

### Dialogs & windows

- `Esc`, `Space` or `Enter` acknowledge most single-button dialogs.
- Research dialog: pick next advance with arrows, confirm with Enter/Space.
- Diplomacy dialog: choose War/Peace, confirm.
- Transport window: click a ship; **Disembark** per cargo unit when adjacent to land.
- Disband dialog: ships with cargo ask **Disband all** / **Cancel**.

### Mouse

- Left click: select unit/city or move adjacent; click windows' buttons.
- Clicking outside an open window dismisses it.

## Cities

- Founded by a Settler on a land tile.
- **Growth**: cities store food; reaching the threshold grows population. Granary/Aqueduct affect growth.
- **Working tiles**: each citizen works one tile in the city radius, yielding food, shields, trade.
- **Production**: cities accumulate shields toward a unit or improvement; when complete, the item is created.
- **Starvation**: if food runs out, population may shrink; a notice is shown.
- **City window** shows population, worked tiles, production progress, improvements, trade routes.

### City improvements

| Improvement | Cost (shields) | Upkeep (gold) | Requires          | Effect                                    |
| ----------- | -------------- | ------------- | ----------------- | ----------------------------------------- |
| Barracks    | 15             | 1             | —                 | New units produced as veterans            |
| City Walls  | 15             | 1             | Masonry           | ×2 defence for all defenders in city      |
| Aqueduct    | 15             | 2             | Construction      | Allows city growth beyond size 2          |
| Temple      | 20             | 1             | Ceremonial Burial | +1 happy citizen                          |
| Library     | 20             | 1             | Writing           | +research (doubles research output)       |
| Marketplace | 20             | 1             | Currency          | Gold income multiplied                    |
| Granary     | 20             | 1             | Pottery           | Food carried over on growth               |
| Courthouse  | 20             | 1             | Code of Laws      | Reduces corruption (standard Civ effect)  |
| Colosseum   | 20             | 4             | Construction      | +3 happy citizens                         |
| Cathedral   | 20             | 3             | Religion          | +4 happy citizens                         |
| Bank        | 20             | 3             | Banking           | Gold income multiplied (with Marketplace) |
| University  | 20             | 3             | University        | Research output multiplied (with Library) |
| Palace      | 30             | 0             | Masonry           | Capital; no upkeep                        |

Happy citizens are twice as productive (shields only). Happiness bonuses stack but never exceed population.

## Economy

- **Shields** (resources) build units/improvements.
- **Food** grows population.
- **Trade** becomes gold and research.
- **Gold income**: worked-tile trade arrows plus special-resource gold, scaled by Marketplace/Bank.
- **Upkeep**: each improvement costs gold per turn; unpaid upkeep may disband a unit.
- **Rush-buy**: `b` in city window pays gold to finish the current build instantly (cost = 2 × remaining shields, min 1).
- **Trade routes**: Caravans entering a city can create lasting routes paying gold per turn to each endpoint.
- **Difficulty** scales rival economies only.

## Research & technology

- Choose a research target; cities contribute research points each turn.
- Advances unlock units, improvements, and further advances (prerequisite tree).
- Research costs vary (e.g., Pottery 15, Magnetism 360). Discovery grants a dialog to pick the next target.
- Diplomats can steal technology from rivals.

## Units

Combat is a race to 10 hit points; each round the attacker hits with probability `a/(a+d)`. Defence bonuses stack: veteran ×1.5, mountain ×2, home city ×1.5, walls ×2, fortified ×1.5.

| Unit     | Attack | Defence | Moves | Cost | Requires         | Notes                                  |
| -------- | ------ | ------- | ----- | ---- | ---------------- | -------------------------------------- |
| Settler  | 0      | 1       | 1     | 60   | —                | Founds cities                          |
| Militia  | 1      | 1       | 1     | 10   | —                |                                        |
| Phalanx  | 1      | 2       | 1     | 15   | Bronze Working   |                                        |
| Legion   | 3      | 1       | 1     | 15   | Iron Working     |                                        |
| Chariot  | 4      | 1       | 1     | 15   | Wheel            |                                        |
| Cavalry  | 2      | 1       | 3     | 20   | Horseback Riding |                                        |
| Knight   | 4      | 2       | 3     | 20   | Chivalry         |                                        |
| Catapult | 6      | 1       | 1     | 20   | Wheel            | Sieges cities (bombards walls)         |
| Diplomat | 0      | 0       | 2     | 20   | Writing          | Enters foreign cities; special actions |
| Caravan  | 0      | 1       | 1     | 30   | Trade            | Creates trade routes                   |
| Trireme  | 1      | 0       | 3     | 30   | Bronze Working   | Carries 2 (coastal build)              |
| Sail     | 1      | 1       | 3     | 20   | Map Making       | Carries 2 (coastal build)              |
| Frigate  | 2      | 2       | 3     | 40   | Magnetism        | Carries 2 (coastal build)              |

- Ships can only be built in coastal cities.
- A unit fortifying on a city tile becomes the garrison (hidden on map).
- Transported units are inert until they disembark onto adjacent land.

## Diplomacy & war

- New civilizations met trigger a diplomacy dialog: declare war or stay at peace.
- At peace, units cannot enter foreign territory.
- Wars allow invasion and city capture. Rivals may declare war (except on Easy).
- City capture: undefended city taken by walking in; defended city requires combat. Population-1 cities are destroyed.
- Diplomats can enter foreign cities (peace or war) and perform: Investigate City, Steal Technology, Industrial Sabotage, Incite a Revolt, Subvert City.
- Establishing embassies/king meetings are not implemented.

## Transport

- Ships carry up to 2 land units. Board by moving a land unit onto a ship.
- Disembark by moving the passenger onto adjacent land (free move).
- Click a ship to open its cargo window; **Disembark** (if adjacent to land) selects that unit to move ashore.

## Map & terrain

- Terrain types affect movement and yields. Special resources (coal, fish, game, gems, gold, horses, oasis, oil) appear on supporting terrain.
- Cities' tiles wear their owner's color; captured cities change color immediately.
- Fog of war hides unexplored tiles.

## Saving & loading

- `S` opens save/load prompt. Enter a path.
- Load via menu or CLI: `civterm file.civ`.

## Advanced systems

- **Upkeep/empty treasury**: unpaid upkeep may disband your costliest unit.
- **Starvation**: hungry cities may lose population; notices appear.
- **Build notices**: cities finishing builds prompt for next orders.
- **Rival movement replay**: after ending turn, rival moves animate.

## Tips

- Fortify defenders in cities; walls make cities very hard to take.
- Use diplomats to weaken rivals before attacking.
- Keep treasury healthy to avoid disbanding.
- Explore early; rival settlers respect all civilizations' city footprints.
