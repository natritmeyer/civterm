# 3 — `game_engine` class diagram

Source: `src/game_engine/*.rs`. Facade `mod.rs` re-exports `Engine`,
`GameView`, `Command`, `Player`, `Event`, `MoveError`, `SettleError`,
`Exploration`, `CityIncome`, `RivalMotion`, `DEFAULT_MAP_WIDTH/HEIGHT`.
One `impl Engine` block per concern file (`movement.rs`, `combat.rs`,
`cities.rs`, `diplomacy.rs`, `research.rs`, `turns.rs`).

```mermaid
classDiagram
    class Engine {
        +Game game
        +int turn
        +PlayerId current_player_index
        +Vec~Event~ events
        +Rng rng
        +Vec~RivalMotion~ motion
        +new(w, h, first, rest)
        +new_random(w, h, first, rest)
        +with_seed(w, h, first, rest, seed)
        +submit(cmd) Vec~Event~
        +drain_rival_motion() Vec~RivalMotion~
        +run_rival_turn()
    }
    class Game {
        +Map map
        +Vec~Player~ players
        +Vec~Unit~ units
        +Vec~City~ cities
        +DISCOVERY_RADIUS = 1
        +spawn_unit(class, loc, owner, home) UnitId
        +remove_unit(id) Option~Unit~
        +add_city(owner, name, loc) CityId
        +reveal_tiles_at(player, loc)
        +reveal_tiles_surrounding_city_at(player, loc)
        +declare_war(a, b)
        +make_peace(a, b)
        +at_war(a, b) bool
        +have_met(a, b) bool
        +sync_cargo(carrier)
        +disband_cargo_of(carrier) int
        +disband_units_homed_to(city) int
        +city_footprint(loc) Vec~Location~
        +auto_assign_work(city)
        +city_breakdown(id) CityIncome
        +research_income(owner) int
        +can_research(owner, adv) bool
    }
    class Player {
        +Civilization civilization
        +STARTING_GOLD = 50
        +new(civ)
        +gold() int
        +eliminated() bool
        +mark_eliminated()
        +advances_made() slice
        +research_progress() int
        +has_advancement(adv) bool
        +can_build(target) bool
        +explored_at(x, y) bool
        +reveal_tiles_at(origin, radius)
    }
    class Command {
        <<enumeration>>
        Move
        Fortify
        Sentry
        Work
        CancelOrder
        FoundCity
        SetProductionTarget
        DeclareWar
        MakePeace
        SetResearchTarget
        EndTurn
    }
    class Event {
        -String message
        +new(msg) Event
        +message() str
    }
    class GameView {
        <<interface>>
        +width() int
        +height() int
        +tile(x, y) Tile
        +units_at(x, y) Vec~Unit~
        +unit(id) Option~Unit~
        +city_at(x, y) Option~City~
        +player_units() Vec~Unit~
        +player_cities() Vec~City~
        +city(id) Option~City~
        +current_player_id() PlayerId
        +city_income(id) CityIncome
        +explored(x, y) bool
        +turn() int
        +year() int
        +gold() int
        +research_progress() int
        +research_cost() Option~int~
        +production_choices(city) Vec~ProductionTarget~
    }
    class CityIncome {
        +int food
        +int resources
        +int trade
        +int gold
        +int research
        +Vec~SpecialResource~ special_resources
    }
    class RivalMotion {
        +UnitId unit
        +Location from
        +Location to
    }
    class Exploration {
        +new(w, h)
        +discovered(x, y) bool
        +reveal_tiles_at(origin, radius)
        +reveal_tiles_surrounding_city_at(origin)
    }
    class MoveError {
        <<enumeration>>
        NoSuchUnit
        NoMovesRemaining
        CannotMoveThere
        CannotCrossLandSeaBorder
        NoShipToBoard
        PeacefulTileOccupied
        +message() String
    }
    class SettleError {
        <<enumeration>>
        NoSuchUnit
        NotASettler
        LandRequired
        Transported
        CityAlreadyHere
        +message() String
    }
    class SaveData {
        +int version
        +int turn
        +PlayerId current_player_index
        +Vec~Event~ events
        +Rng rng
        +Game game
        +Competition competition
        +Difficulty difficulty
        +capture(engine, comp, diff) SaveData
        +into_loaded() LoadedGame
    }
    class LoadedGame {
        +Engine engine
        +Competition competition
        +Difficulty difficulty
    }

    Engine *-- Game : 1 owns
    Engine *-- Event : 0..* log
    Engine *-- RivalMotion : 0..* transient replay
    Engine ..|> GameView : implements
    Game *-- Player : 1..* players
    Game *-- Unit : 0..* units
    Game *-- City : 0..* cities
    Game *-- Map : 1 map
    Game ..> CityIncome : city_breakdown()
    Player *-- Exploration : 1 fog bitmap
    Player ..> Civilization : civilization
    Player ..> Advancement : research state
    Engine ..> Command : submit() dispatches
    Engine ..> MoveError : move_unit errors
    Engine ..> SettleError : found_city errors
    SaveData *-- Game : nested game
    SaveData ..> LoadedGame : into_loaded()
    SaveData ..> Engine : capture() clones
    RivalMotion ..> UnitId : unit
    RivalMotion ..> Location : from/to
```

## Dispatch table (`Engine::submit` → concern module)

| `Command` | Owner file | Key callees |
|---|---|---|
| `Move` | `movement.rs` | `try_board`, `meet_contacts_within`, `resolve_move_combat` (`combat.rs`), `capture_city` (`combat.rs`) |
| `Fortify` / `Sentry` / `Work` / `CancelOrder` | `movement.rs` | `owned_unit_mut`, terrain `supports()` gates |
| `FoundCity` / `SetProductionTarget` | `cities.rs` | `Game::add_city`, `auto_assign_work`, `player.can_build` |
| `DeclareWar` / `MakePeace` | `diplomacy.rs` | `Game::declare_war` / `make_peace` |
| `SetResearchTarget` | `research.rs` | `Game::can_research`, `set_research_target` |
| `EndTurn` | `turns.rs` + `rival_player_engine.rs` | `advance_to_next_player`, `begin_turn`, `run_rival_turn`, `drain_rival_motion` |

`Engine.motion` is transient: rebuilt fresh in `into_loaded`, never serialized into `SaveData`.
