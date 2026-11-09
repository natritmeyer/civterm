# CivTerm — Economy

Reference for gold, upkeep, rush-buying, and trade income. See
`docs/CITY_IMPROVEMENTS.md` and `docs/TRADE_ROUTES.md`.

## Gold income

- A city's gold take is its raw tax base scaled by Marketplace/Bank:
  `City::gold_income(raw)`.
- Raw tax base = worked-tile trade arrows + special-resource gold
  (`Game::city_gold_income`).
- Banked once per city in `process_cities` via `Player::add_gold`
  (saturating).
- Every owner earns (human and rival); difficulty scales a rival's take.

## Upkeep

- Each improvement costs gold per turn (`CityImprovement::upkeep`).
- `City::upkeep()` sums a city's bill; owners charged the sum over all
  cities once per round, after gold income.
- `Engine::charge_upkeep` pays what it can; shortfall clamps treasury to
  zero and disbands the costliest unit (ties: highest unit id).
- A disbanded carrier takes its cargo with it.
- Only the human's shortfall is announced (`UpkeepNotice` window).

## Rush-buying

- `City::rush_cost()` = `2 × (cost − stored)`, floored at 1 gold.
- `Engine::buy_production(city)` is the single command behind the city
  window's `b` key and Buy button.
- Re-checks ownership and affordability; rejections are event-only.
- `City::rush_production` fills the store to the target's cost and
  completes via `complete_if_ready`, so a rushed unit wears a Barracks'
  veteran stripe like any other.
- A city may rush-buy at most once per turn (`rushed_this_turn` flag,
  reset each turn by `tick`).

## Trade income

- See `docs/TRADE_ROUTES.md`: each endpoint pays its gold per round,
  banked before upkeep.

## Difficulty scaling

- Rival economies scaled by `Scale` (production, research, growth, gold);
  human never scaled. See `docs/DIFFICULTY.md`.