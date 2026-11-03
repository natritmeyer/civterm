# CivTerm — Difficulty

Reference for the difficulty levels and their effects. See
`docs/INSTRUCTIONS.md` for the player-facing table.

## Levels

| Level  | Starting gold | Rivals declare war? | Rivals attack at even odds? | Rival win-chance floor | Rival economy |
| ------ | ------------- | ------------------- | --------------------------- | ---------------------- | ------------- |
| Easy   | 50            | No                  | No                          | 10%                    | 3/4 prod, 3/4 research, 3/2 growth food, 3/4 gold |
| Normal | 0             | Yes                 | No                          | 10%                    | Identity      |
| Hard   | 0             | Yes                 | Yes                         | 0%                     | 5/4 prod, 5/4 research, 3/4 growth food, 5/4 gold |

## Policy knobs (methods on `Difficulty`)

- `starting_gold`: Easy 50, Normal/Hard 0.
- `rivals_initiate_war`: false only on Easy.
- `rivals_attack_at_even_odds`: true only on Hard.
- `rival_minimum_win_chance`: Hard 0, otherwise 10.
- `rival_economy()`: the four `Scale` ratios.

## Application

- `Engine::set_difficulty` stores the level and stakes the human's opening
  treasury (player 0 gets `starting_gold`, rivals 0). Construction-time only.
- `Engine::economy_scales(owner)` returns identity for player 0, the
  difficulty's `rival_economy()` otherwise.
- Rival war policy has two call sites: `rivals_initiate_war` and the edge
  test in `rival_player_engine`.

## Rival economy scaling

`Scale::apply` never cuts a nonzero yield to zero, so a size-1 rival city
still banks a shield, a research point, and a coin. Easy widens the growth
box (slower) and leaves the rival poorer; Hard narrows it and fattens the
treasury.

## Persistence

- `SaveData` holds `difficulty`; `into_loaded` threads it into both
  `LoadedGame` and the rebuilt `Engine`.