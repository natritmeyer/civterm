# 2 — `model` domain class diagram

Source: `src/model/**`. `model/mod.rs` is a facade only. Entities are linked by
small index-backed ID values (`UnitId`, `CityId`, `PlayerId`) resolved by linear
scan — no pointers, no hash maps. `MapGenerator` is the only type that touches
`utils::Rng` (outside `model`).

```mermaid
classDiagram
    class Map {
        +int width
        +int height
        +new(width, height)
        +tile_at(loc) Tile
        +tile_at_mut(loc) Tile
        +destination(from, dir) Option~Location~
        +render_ascii() String
    }
    class Tile {
        +Terrain terrain
        +place_resource(r) Result
        +irrigate() Result
        +mine() Result
        +build_road() Result
        +can_build(imp) bool
        +apply_improvement(imp) Result
        +is_irrigated() bool
        +is_mined() bool
        +has_road() bool
        +resource() Option~SpecialResource~
        +yields_food() int
        +yields_resources() int
        +yields_trade() int
    }
    class Location {
        +int x
        +int y
        +new(x, y)
        +is_adjacent(other) bool
    }
    class Direction {
        <<enumeration>>
        N
        NE
        E
        SE
        S
        SW
        W
        NW
        +delta() tuple
    }
    class MapGenerator {
        +new(seed)
        +with_rng(rng)
        +generate(w, h) Map
        +fill_enclosed_water_with_grass(map)
        +grow_continents(map, n)
    }
    class Terrain {
        <<enumeration>>
        Ocean
        Grassland
        Plains
        Forest
        Hills
        Mountain
        Desert
        Tundra
        Swamp
        Jungle
        +movement_cost() int
        +is_water() bool
        +is_land() bool
        +supports(imp) bool
    }
    class MovementCategory {
        <<enumeration>>
        Open
        Dense
        Mountain
        +movement_cost() int
    }
    class SpecialResource {
        <<enumeration>>
        Coal
        Fish
        Game
        Gems
        Gold
        Horses
        Oasis
        Oil
    }
    class TerrainImprovement {
        <<enumeration>>
        Irrigation
        Mine
        Road
        +work_turns() int
    }
    class Unit {
        +UnitClass unit_class
        +Location location
        +new(class, loc, owner, home, id)
        +id() UnitId
        +owner() PlayerId
        +home_city() CityId
        +order() UnitOrder
        +moves_remaining() int
        +aboard() Option~UnitId~
        +is_transported() bool
        +board(carrier)
        +disembark()
        +spend_moves(n)
        +spend_turn()
        +restore_moves()
        +fortify()
        +sentry()
        +work(imp)
        +advance_work()
        +cancel_order()
        +promote()
    }
    class UnitClass {
        <<enumeration>>
        Settler
        Militia
        Phalanx
        Legion
        Cavalry
        Chariot
        Knight
        Catapult
        Diplomat
        Caravan
        Trireme
        Sail
        Frigate
        +attack() int
        +defence() int
        +moves() int
        +carry_capacity() int
        +can_found_city() bool
        +can_travel_water() bool
        +required_advancement() Option~Advancement~
    }
    class UnitOrder {
        <<enumeration>>
        Idle
        Fortified
        Sentried
        Improving
    }
    class UnitId {
        +new(i) UnitId
        +index() int
    }
    class City {
        +String name
        +Location location
        +new(name, loc, owner, id)
        +id() CityId
        +owner() PlayerId
        +change_owner(p)
        +population() int
        +grow()
        +shrink()
        +improvements() slice
        +add_improvement(imp)
        +set_production(t)
        +production_target() Option~ProductionTarget~
        +worked_tiles() slice
        +add_worked_tile(loc)
        +tick(food, res) CityTick
        +research() int
    }
    class CityImprovement {
        <<enumeration>>
        Aqueduct
        Bank
        Barracks
        Cathedral
        CityWalls
        Colosseum
        Courthouse
        Granary
        Library
        Marketplace
        Palace
        Temple
        University
        +required_advancement() Option~Advancement~
    }
    class ProductionTarget {
        <<enumeration>>
        Unit
        Improvement
        +resource_cost() int
    }
    class CityTick {
        +int produced
        +bool grew
        +Option~ProductionTarget~ completed
        +bool starving
    }
    class CityId {
        +new(i) CityId
        +index() int
    }
    class Civilization {
        <<enumeration>>
        American
        Aztec
        Babylonian
        Chinese
        Egyptian
        English
        French
        German
        Greek
        Indian
        Mongol
        Roman
        Russian
        Zulu
        +display_name() str
        +capital_name() str
        +ruler() Ruler
    }
    class Ruler {
        <<enumeration>>
        AbrahamLincoln
        Montezuma
        Hammurabi
        MaoZedong
        Ramses
        QueenElizabethI
        Napoleon
        FrederickTheGreat
        Alexander
        Gandhi
        GenghisKhan
        JuliusCaesar
        Stalin
        Shaka
    }
    class PlayerId {
        +new(i) PlayerId
        +index() int
    }
    class Advancement {
        <<enumeration>>
        Alphabet
        BronzeWorking
        CeremonialBurial
        Construction
        Pottery
        Writing
        +prerequisites() slice
        +cost() int
    }
    class Competition {
        +new(rivals)
        +rivals() int
        +total_civilizations() int
    }
    class Difficulty {
        <<enumeration>>
        Easy
        Normal
        Hard
    }

    Map *-- Tile : 1 contains width x height
    Map ..> Location : indexed by
    Map ..> Direction : destination()
    MapGenerator ..> Map : generates
    MapGenerator ..> Rng : owns utils Rng
    Tile *-- Terrain : terrain by value
    Tile ..> SpecialResource : 0..1 resource
    Tile ..> TerrainImprovement : can_build / apply
    Terrain ..> MovementCategory : movement_class()
    Terrain ..> TerrainImprovement : supports()
    Terrain ..> SpecialResource : supports_resource()
    Unit *-- UnitClass : class by value
    Unit *-- UnitOrder : order by value
    UnitOrder ..> TerrainImprovement : Improving payload
    Unit ..> UnitId : id + aboard carrier
    Unit ..> PlayerId : owner
    Unit ..> CityId : home_city
    Unit ..> Location : location copy
    City *-- CityImprovement : 0..* owned
    City *-- ProductionTarget : 0..1 active
    City ..> CityId : id
    City ..> PlayerId : owner
    City ..> Location : location + worked tiles
    City ..> CityTick : tick() returns
    CityTick ..> ProductionTarget : 0..1 completed
    ProductionTarget ..> UnitClass : Unit payload
    ProductionTarget ..> CityImprovement : Improvement payload
    UnitClass ..> Advancement : required_advancement()
    CityImprovement ..> Advancement : required_advancement()
    Civilization ..> Ruler : 1-1 ruler()
    Rng ..> Map : fills tiles
```

> `Advancement` shows 6 of 34 variants for brevity; the full list
> (`Alphabet` … `Writing`) is in `src/model/advancements/advancement.rs`.
> `City.worked` always includes the centre tile; max 21 tiles (radius-2 diamond
> minus corners) — see `Game::city_footprint` in `game_engine`.
