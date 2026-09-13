# Ubiquitous language

How the acceptance tests under `tests/features/` speak about the game. Each
phrase maps to exactly one step definition in `tests/world/steps.rs`.

Words in braces are cucumber parameters:

- `{civ}` — one of `English`, `Zulu`, `Roman`.
- `{dir}` — one of `north`, `north-east`, `east`, `south-east`, `south`,
  `south-west`, `west`, `north-west`.
- `{year}` — one of `4000 BC`, `3000 BC`, `1000 BC`, `975 BC`, `1 AD`.
- `{advancement}` — `Construction` or `Wheel`.
- `{improvement}` — `a road`, `a mine`, `an irrigation` (the work that is
  started).
- `{improved}` — `roaded`, `mined`, `irrigated` (the state a tile wears).
- `{landscape}` — `open land`, `dense terrain`, `open water`.
- `{start}` — the starting condition a scenario is seeded with (below).
- `{word}` — a quoted single token (a city name or a unit class such as
  `militia`).
- `{int}` — a whole number.

## Starting conditions

A scenario that begins with "the English settle on …" searches the seed range
for a map whose starting tile satisfies the stated condition:

- `open land to the north` — a cheap (cost 1) land tile north of the settler.
- `dense terrain to the north` — a cost-2 land tile north of the settler.
- `sea to the west` — open water directly west of the settler.
- `fertile land` — the starting tile yields 3 food.
- `plentiful land` — the starting tile yields 2 food.
- `land that offers no food` — the starting tile yields no food.
- `rocky hills` — a mine can be built on the starting tile.
- `fertile grassland` — an irrigation can be built on the starting tile.
- `forest land` — the starting tile yields 1 food and 2 resources.

## Steps

### Setup

- Given the {civ} settle on {start} — build a solo game for that civilization
  on the seed that satisfies the starting condition.
- Given a new game with the {civ}, the {civ} and the {civ} — a three
  civilization game; the first civilization named moves first.

### Orders (When)

- the {civ} move {dir} — the settler steps toward `dir`.
- the {civ} found a city named {word} — the settler gives up its life for a
  capital.
- exactly {int} turn ends / exactly {int} turns end — pass the stated number
  of full turns.
- the {civ} begin researching the {advancement}.
- the {civ} declare war on the {civ} / the {civ} make peace with the {civ}.
- the {civ} fortify the settler / the {civ} put the settler on sentry.
- the {civ} begin building {improvement}.
- the {civ} cancel the settler's order.
- the {civ} set {word} to produce a {word} — point the founded city at a unit
  class (the second `{word}`).

### Reports (Then)

- the year is {year} / it is turn {int} — the calendar.
- the treasury holds {int} gold — the current player's purse.
- the map is {int} tiles wide and {int} tiles tall.
- the {civ} starting tile is explored / the land around the {civ} starting
  tile is explored / the land two tiles to the east of the {civ} starting
  tile is explored — the map's fog of war around the spawn.
- the {civ} settler is one tile {dir} of where it started / the {civ} settler
  stands where it started — where a move deposited the settler.
- the {civ} moving {dir} is reported — the move was announced.
- the {civ} settler's tile is {landscape}.
- the {civ} settler has no moves remaining.
- the {civ} report that the sea stops the settler — the land/sea border.
- the {civ} report that the settler is spent — no moves left to spend.
- the city of {word} stands where the settler stood — founding consumed the
  settler exactly there.
- the {civ} have no settler left.
- the {civ} beginning research on {advancement} is reported / the {civ}
  discovering {advancement} is reported.
- the {civ} are researching {advancement} / the {civ} are researching
  nothing / the {civ} research progress is {int}.
- the {civ} have {advancement} / the {civ} do not have {advancement}.
- the city of {word} grows to size {int} / the {civ} report that the city of
  {word} grows / the city of {word} is still size {int} / the {civ} report
  that the city of {word} is starving — population reports.
- the city of {word} begins producing a {word} is reported / the city of
  {word} is producing a {word} / the city of {word} producing a {word} is
  reported — production reports.
- the {civ} now have a {word} unit / the {civ} do not have a {word} unit.
- the city of {word} is owned by the {civ}.
- the {civ} declaring war on the {civ} is reported / the {civ} report that
  they are already at war with the {civ} / the {civ} making peace with the
  {civ} is reported / the {civ} report that they are already at peace with
  the {civ} / the {civ} declaring war on themselves is refused / the {civ}
  making peace with themselves is refused — diplomacy reports.
- the {civ} settler is fortified / the {civ} settler is on sentry.
- the {civ} settler beginning work on {improvement} is reported / the {civ}
  settler finishing {improvement} is reported.
- the tile the {civ} settler stands on is {improved} / … is not {improved}.
- the {civ} settler's order is cancelled.