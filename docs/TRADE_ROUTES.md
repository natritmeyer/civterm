# CivTerm — Trade Routes

Reference for Caravan-based trade routes. See `docs/UNITS.md` (Caravan) and
`docs/ECONOMY.md`.

## Opening a route

- A Caravan walking into a city triggers `try_establish_trade_route`.
- On success the caravan is consumed (removed); the route is created.
- Refused if the destination already has `MAX_TRADE_ROUTES` (3) routes
  ending there; the caravan survives and an event explains the refusal.
- At war the crossing is refused (a route is trade, not a provocation).
- A Caravan still `enters_to_conquer`, so at war on an undefended city it
  captures like any other unit.

## Route value

Quoted once on arrival (`trade_route_value`):

- Base 1
- + wrapped Chebyshev distance to the far city / 4
- + far city's population
- + 2 per distinct special resource worked by either endpoint

Constants live in `trade.rs`.

## Payout

- Every endpoint pays its gold to whoever holds it, once per round.
- Banked in `process_cities` before upkeep, so a route can cover upkeep.
- A link between two cities of one owner pays that owner twice.
- The human's take is announced as an event.

## Death

- A route dies with its city: an overrun city's routes are purged
  (`remove_trade_routes_touching`), so no route outlives an endpoint.
- A city merely changing hands keeps its routes, which now pay the new owner.

## Display

- The city window's right column shows a "Trade Routes" section: each row
  names the far city and the gold it pays.
- Foreign (investigated) cities show their routes too.