use std::fmt;
use std::str::FromStr;

use civterm::game_engine::{Command, Engine, GameView, Player};
use civterm::model::advancements::Advancement;
use civterm::model::cartography::{Direction, Location, Tile};
use civterm::model::cities::CityId;
use civterm::model::civilizations::Civilization;
use civterm::model::geography::TerrainImprovement;
use civterm::model::units::UnitClass;

/// Why a step argument could not be turned into the value it names.
#[derive(Debug)]
pub struct ArgError(pub String);

impl fmt::Display for ArgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A civilization's name as it appears in a feature file.
#[derive(Debug, cucumber::Parameter)]
#[param(regex = "English|Zulu|Roman", name = "civ")]
pub struct CivArg(pub Civilization);

impl FromStr for CivArg {
    type Err = ArgError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "English" => Ok(CivArg(Civilization::English)),
            "Zulu" => Ok(CivArg(Civilization::Zulu)),
            "Roman" => Ok(CivArg(Civilization::Roman)),
            other => Err(ArgError(format!("unknown civilization {other:?}"))),
        }
    }
}

/// A compass direction as it appears in a feature file.
#[derive(Debug, cucumber::Parameter)]
#[param(
    regex = "north|north-east|east|south-east|south|south-west|west|north-west",
    name = "dir"
)]
pub struct DirArg(pub Direction);

impl FromStr for DirArg {
    type Err = ArgError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "north" => Ok(DirArg(Direction::N)),
            "north-east" => Ok(DirArg(Direction::NE)),
            "east" => Ok(DirArg(Direction::E)),
            "south-east" => Ok(DirArg(Direction::SE)),
            "south" => Ok(DirArg(Direction::S)),
            "south-west" => Ok(DirArg(Direction::SW)),
            "west" => Ok(DirArg(Direction::W)),
            "north-west" => Ok(DirArg(Direction::NW)),
            other => Err(ArgError(format!("unknown direction {other:?}"))),
        }
    }
}

/// The report of a single turn's time as it appears in a feature file.
#[derive(Debug, cucumber::Parameter)]
#[param(regex = "4000 BC|3000 BC|1000 BC|975 BC|1 AD", name = "year")]
pub struct YearArg(pub i32);

impl FromStr for YearArg {
    type Err = ArgError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "4000 BC" => Ok(YearArg(-4000)),
            "3000 BC" => Ok(YearArg(-3000)),
            "1000 BC" => Ok(YearArg(-1000)),
            "975 BC" => Ok(YearArg(-975)),
            "1 AD" => Ok(YearArg(0)),
            other => Err(ArgError(format!("unknown year {other:?}"))),
        }
    }
}

/// An advancement as it appears in a feature file.
#[derive(Debug, cucumber::Parameter)]
#[param(regex = "Construction|Wheel", name = "advancement")]
pub struct AdvancementArg(pub Advancement);

impl FromStr for AdvancementArg {
    type Err = ArgError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Construction" => Ok(AdvancementArg(Advancement::Construction)),
            "Wheel" => Ok(AdvancementArg(Advancement::Wheel)),
            other => Err(ArgError(format!("unknown advancement {other:?}"))),
        }
    }
}

/// A working improvement as it appears in a feature file.
#[derive(Debug, cucumber::Parameter)]
#[param(regex = "a road|a mine|an irrigation", name = "improvement")]
pub struct TerrainImprovementArg(pub TerrainImprovement);

impl FromStr for TerrainImprovementArg {
    type Err = ArgError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "a road" => Ok(TerrainImprovementArg(TerrainImprovement::Road)),
            "a mine" => Ok(TerrainImprovementArg(TerrainImprovement::Mine)),
            "an irrigation" => Ok(TerrainImprovementArg(TerrainImprovement::Irrigation)),
            other => Err(ArgError(format!("unknown improvement {other:?}"))),
        }
    }
}

/// The state an improved tile carries as it appears in a feature file.
#[derive(Debug, cucumber::Parameter)]
#[param(regex = "roaded|mined|irrigated", name = "improved")]
pub struct TerrainStateArg(pub TerrainImprovement);

impl FromStr for TerrainStateArg {
    type Err = ArgError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "roaded" => Ok(TerrainStateArg(TerrainImprovement::Road)),
            "mined" => Ok(TerrainStateArg(TerrainImprovement::Mine)),
            "irrigated" => Ok(TerrainStateArg(TerrainImprovement::Irrigation)),
            other => Err(ArgError(format!("unknown improvement state {other:?}"))),
        }
    }
}

/// The kind of ground a tile offers, as it appears in a feature file.
#[derive(Debug, Clone, Copy)]
pub enum Landscape {
    OpenLand,
    DenseTerrain,
    OpenWater,
}

#[derive(Debug, cucumber::Parameter)]
#[param(regex = "open land|dense terrain|open water", name = "landscape")]
pub struct LandscapeArg(pub Landscape);

impl FromStr for LandscapeArg {
    type Err = ArgError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "open land" => Ok(LandscapeArg(Landscape::OpenLand)),
            "dense terrain" => Ok(LandscapeArg(Landscape::DenseTerrain)),
            "open water" => Ok(LandscapeArg(Landscape::OpenWater)),
            other => Err(ArgError(format!("unknown landscape {other:?}"))),
        }
    }
}

/// The starting world a scenario asks for. Each starting condition searches
/// the seed range for a map whose starting tile satisfies it.
#[derive(Debug, Clone, Copy)]
pub enum StartingCondition {
    OpenLandNorth,
    DenseTerrainNorth,
    SeaWest,
    FertileLand,
    PlentifulLand,
    BarrenLand,
    RockyHills,
    FertileGrassland,
    ForestLand,
}

impl StartingCondition {
    /// A seed whose starting world satisfies this starting condition.
    pub fn seed(self) -> u64 {
        self.cached_seed()
    }

    /// The seed each starting condition first resolves to, cached so a scenario
    /// rerun on the same starting condition does not re-search the range.
    fn cached_seed(&self) -> u64 {
        static CACHE: [std::sync::OnceLock<u64>; 9] = [const { std::sync::OnceLock::new() }; 9];
        let slot = &CACHE[self.index()];
        *slot.get_or_init(|| self.find_seed())
    }

    fn index(&self) -> usize {
        match self {
            StartingCondition::OpenLandNorth => 0,
            StartingCondition::DenseTerrainNorth => 1,
            StartingCondition::SeaWest => 2,
            StartingCondition::FertileLand => 3,
            StartingCondition::PlentifulLand => 4,
            StartingCondition::BarrenLand => 5,
            StartingCondition::RockyHills => 6,
            StartingCondition::FertileGrassland => 7,
            StartingCondition::ForestLand => 8,
        }
    }

    fn find_seed(&self) -> u64 {
        for seed in 1..=4096 {
            let (engine, start, city) = found(seed);
            if self.matches(&engine, start, city) {
                return seed;
            }
        }
        panic!("no seed in 1..=4096 satisfies the starting condition {self:?}");
    }

    fn matches(self, engine: &Engine, start: Location, city: Option<CityId>) -> bool {
        match self {
            StartingCondition::OpenLandNorth => neighbour(engine, start, Direction::N)
                .is_some_and(|tile| tile.terrain.is_land() && tile.terrain.movement_cost() == 1),
            StartingCondition::DenseTerrainNorth => neighbour(engine, start, Direction::N)
                .is_some_and(|tile| tile.terrain.is_land() && tile.terrain.movement_cost() == 2),
            StartingCondition::SeaWest => {
                neighbour(engine, start, Direction::W).is_some_and(|tile| tile.terrain.is_water())
            }
            StartingCondition::FertileLand => {
                city.is_some_and(|id| engine.city_income(id).food == 3)
            }
            StartingCondition::PlentifulLand => {
                city.is_some_and(|id| engine.city_income(id).food == 2)
            }
            StartingCondition::BarrenLand => {
                city.is_some_and(|id| engine.city_income(id).food == 0)
            }
            StartingCondition::RockyHills => {
                tile(engine, start).can_build(TerrainImprovement::Mine)
            }
            StartingCondition::FertileGrassland => {
                tile(engine, start).can_build(TerrainImprovement::Irrigation)
            }
            StartingCondition::ForestLand => city.is_some_and(|id| {
                let income = engine.city_income(id);
                income.food == 2 && income.resources == 2
            }),
        }
    }
}

#[derive(Debug, cucumber::Parameter)]
#[param(
    regex = "open land to the north|dense terrain to the north|sea to the west|fertile land|plentiful land|land that offers no food|rocky hills|fertile grassland|forest land",
    name = "start"
)]
pub struct StartingConditionArg(pub StartingCondition);

impl FromStr for StartingConditionArg {
    type Err = ArgError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "open land to the north" => Ok(StartingConditionArg(StartingCondition::OpenLandNorth)),
            "dense terrain to the north" => {
                Ok(StartingConditionArg(StartingCondition::DenseTerrainNorth))
            }
            "sea to the west" => Ok(StartingConditionArg(StartingCondition::SeaWest)),
            "fertile land" => Ok(StartingConditionArg(StartingCondition::FertileLand)),
            "plentiful land" => Ok(StartingConditionArg(StartingCondition::PlentifulLand)),
            "land that offers no food" => Ok(StartingConditionArg(StartingCondition::BarrenLand)),
            "rocky hills" => Ok(StartingConditionArg(StartingCondition::RockyHills)),
            "fertile grassland" => Ok(StartingConditionArg(StartingCondition::FertileGrassland)),
            "forest land" => Ok(StartingConditionArg(StartingCondition::ForestLand)),
            other => Err(ArgError(format!("unknown starting condition {other:?}"))),
        }
    }
}

/// A solo English game on `seed` and where its settler started.
pub fn solo(seed: u64) -> (Engine, Location) {
    let mut engine = Engine::with_seed(
        civterm::game_engine::DEFAULT_MAP_WIDTH,
        civterm::game_engine::DEFAULT_MAP_HEIGHT,
        Player::new(Civilization::English),
        Vec::new(),
        seed,
    );
    engine.populate_starting_world();
    let location = engine.player_units()[0].location;
    (engine, location)
}

/// A solo English game on `seed`, its settler's starting tile, and the city
/// founded there, readied for income probing.
fn found(seed: u64) -> (Engine, Location, Option<CityId>) {
    let (mut engine, location) = solo(seed);
    let unit = engine.player_units()[0].id();
    engine.submit(Command::FoundCity {
        unit,
        name: "London".to_string(),
    });
    let city = engine.player_cities()[0].id();
    (engine, location, Some(city))
}

/// The unit class a feature file means by `word`.
pub fn unit_class(word: &str) -> UnitClass {
    match word {
        "militia" => UnitClass::Militia,
        other => panic!("unknown unit class {other:?}"),
    }
}

/// `from` shifted `distance` tiles toward `direction`, wrapping east/west and
/// staying on the map north/south, mirroring `Map::destination`.
pub fn translate(
    width: usize,
    height: usize,
    from: Location,
    direction: Direction,
    distance: u32,
) -> Option<Location> {
    let (dx, dy) = direction.delta();
    let (dx, dy) = (dx * distance as isize, dy * distance as isize);
    let x = (from.x as isize + dx).rem_euclid(width as isize) as u16;
    let y = from.y as isize + dy;
    (y >= 0 && y < height as isize).then(|| Location::new(x, y as u16))
}

/// The tile at `location` on the given engine's map.
fn tile(engine: &Engine, location: Location) -> &Tile {
    engine.tile(location.x as usize, location.y as usize)
}

/// The tile one step from `from` toward `direction`, if it is on the map.
fn neighbour(engine: &Engine, from: Location, direction: Direction) -> Option<&Tile> {
    translate(engine.width(), engine.height(), from, direction, 1)
        .map(|location| tile(engine, location))
}
