# CivTerm — Terrain & Geography

Reference for terrain types, movement, yields, supported improvements and
special resources. Yields are per worked tile; improvements add to them.

## Terrain table

| Terrain   | Move cost | Food | Shields | Trade | Water? | Irrigation | Mine | Road |
| --------- | --------- | ---- | ------- | ----- | ------ | ---------- | ---- | ---- |
| Ocean     | 1         | 2    | 0       | 2     | Yes    | —          | —    | —    |
| Grassland | 1         | 2    | 0       | 1     | No     | Yes        | —    | Yes  |
| Plains    | 1         | 2    | 1       | 0     | No     | Yes        | —    | Yes  |
| Forest    | 2         | 1    | 2       | 0     | No     | —          | —    | Yes  |
| Hills     | 2         | 1    | 1       | 0     | No     | —          | Yes  | Yes  |
| Mountain  | 3         | 0    | 1       | 1     | No     | —          | Yes  | Yes  |
| Desert    | 1         | 0    | 1       | 1     | No     | Yes        | Yes  | Yes  |
| Tundra    | 1         | 1    | 0       | 1     | No     | —          | —    | Yes  |
| Swamp     | 2         | 1    | 0       | 1     | No     | Yes        | —    | Yes  |
| Jungle    | 2         | 1    | 1       | 0     | No     | Yes        | —    | Yes  |

Move cost is in movement points (1–3). Roads reduce a road-to-road step to
1/3 of a point.

## Movement categories

| Category  | Cost | Terrains                                    |
| --------- | ---- | ------------------------------------------- |
| Open      | 1    | Ocean, Grassland, Plains, Desert, Tundra    |
| Dense     | 2    | Forest, Hills, Swamp, Jungle                |
| Mountain  | 3    | Mountain                                    |

## Terrain improvements

| Improvement | Supported on                              | Effect on yields          |
| ----------- | ----------------------------------------- | ------------------------- |
| Irrigation  | Grassland, Plains, Desert, Swamp, Jungle   | +1 food                   |
| Mine        | Hills, Mountain, Desert                   | +1 shields (resources)    |
| Road        | All land                                  | +1 trade                  |

- Irrigation and Mine take 2 and 3 turns to build respectively; Road takes 2.
- A tile may hold irrigation, a mine, and a road simultaneously.
- Roads also grant the movement bonus (road-to-road = 1/3 point).

## Special resources

Each supporting terrain has a 10% chance of receiving one resource at map
generation. A tile holds zero or one resource.

| Resource | Terrain      | Food | Shields | Trade | Gold |
| -------- | ------------ | ---- | ------- | ----- | ---- |
| Coal     | Hills        | 0    | 1       | 0     | 0    |
| Fish     | Ocean        | 1    | 0       | 0     | 0    |
| Game     | Forest, Tundra | 1  | 0       | 0     | 0    |
| Gems     | Jungle       | 0    | 0       | 2     | 0    |
| Gold     | Mountain     | 0    | 0       | 2     | 2    |
| Horses   | Plains       | 0    | 1       | 0     | 0    |
| Oasis    | Desert       | 3    | 0       | 0     | 0    |
| Oil      | Swamp        | 0    | 2       | 0     | 0    |

- Special-resource gold (Gold on a mountain) is banked as gold income in
  addition to trade arrows.
- Resources outrank improvements in the tile watermark display.

## Coastal cities

A city borders water if at least one of its 8 neighbours is ocean. Only such
cities may build naval units.