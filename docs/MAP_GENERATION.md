# CivTerm — Map Generation

Reference for world generation. See `docs/TERRAIN.md`.

## Order of operations

`MapGenerator::generate` settles terrain first, special resources last:

1. Ocean.
2. Tundra borders.
3. Seeded plains.
4. Continents.
5. Enclosed water filled in.
6. `place_special_resources` (last).

Resources come last so a lake that fills in as grassland cannot keep a fish
it should never have had.

## Terrain

- Map dimensions: 120 × 80 by default.
- 7 continents (increased from 4).

## Special resources

- A tile whose terrain supports a resource has a 10% chance
  (`RESOURCE_CHANCE_PERCENT`) of receiving it; unsupporting terrain gets
  nothing.
- `Terrain::supports_resource` is the single terrain → resource mapping.
- `Tile.resource` is an `Option`, so a tile holds zero or one resource.

## City footprint overlap

Rival settlers pick sites against the union of all civilizations' 21-tile
working grids (see `docs/RIVAL_AI.md`). Overlap tiers:

- Open: ≤ 3 shared tiles (≥ Chebyshev 4 off every city).
- Acceptable: ≤ 6 shared tiles (≥ Chebyshev 3).
- Frontier / crowded fallbacks for explored-only maps.