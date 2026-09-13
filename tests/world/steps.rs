use civterm::game_engine::{Command, GameView};
use civterm::model::cartography::Direction;
use civterm::model::cities::ProductionTarget;
use civterm::model::geography::TerrainImprovement;
use civterm::model::units::UnitOrder;
use cucumber::{given, then, when};

use crate::world::CivWorld;
use crate::world::args::{
    AdvancementArg, CivArg, DirArg, Landscape, LandscapeArg, StartingConditionArg,
    TerrainImprovementArg, TerrainStateArg, YearArg, translate, unit_class,
};

#[given(expr = "the {civ} settle on {start}")]
async fn the_civ_settle_on(world: &mut CivWorld, civ: CivArg, condition: StartingConditionArg) {
    world.settle(civ.0, condition.0);
}

#[given(expr = "a new game with the {civ}, the {civ} and the {civ}")]
async fn a_new_game_with_three(world: &mut CivWorld, first: CivArg, second: CivArg, third: CivArg) {
    world.start_trio(first.0, second.0, third.0);
}

#[when(expr = "the {civ} move {dir}")]
async fn the_civ_move(world: &mut CivWorld, _civ: CivArg, dir: DirArg) {
    world.submit(Command::Move {
        unit: world.settler_id(),
        direction: dir.0,
    });
}

#[when(expr = "the {civ} found a city named {word}")]
async fn the_civ_found_a_city(world: &mut CivWorld, _civ: CivArg, name: String) {
    world.submit(Command::FoundCity {
        unit: world.settler_id(),
        name,
    });
    world.record_city();
}

#[when(expr = "exactly {int} turn ends")]
async fn exactly_one_turn(world: &mut CivWorld, turns: u32) {
    world.end_turns(turns);
}

#[when(expr = "exactly {int} turns end")]
async fn exactly_some_turns(world: &mut CivWorld, turns: u32) {
    world.end_turns(turns);
}

#[when(expr = "the {civ} begin researching the {advancement}")]
async fn the_civ_begin_researching(
    world: &mut CivWorld,
    _civ: CivArg,
    advancement: AdvancementArg,
) {
    world.submit(Command::SetResearchTarget {
        advancement: advancement.0,
    });
}

#[when(expr = "the {civ} declare war on the {civ}")]
async fn the_civ_declare_war(world: &mut CivWorld, _aggressor: CivArg, victim: CivArg) {
    world.submit(Command::DeclareWar {
        opponent: world.player_id(victim.0),
    });
}

#[when(expr = "the {civ} make peace with the {civ}")]
async fn the_civ_make_peace(world: &mut CivWorld, _aggressor: CivArg, victim: CivArg) {
    world.submit(Command::MakePeace {
        opponent: world.player_id(victim.0),
    });
}

#[when(expr = "the {civ} fortify the settler")]
async fn the_civ_fortify(world: &mut CivWorld, _civ: CivArg) {
    world.submit(Command::Fortify {
        unit: world.settler_id(),
    });
}

#[when(expr = "the {civ} put the settler on sentry")]
async fn the_civ_sentry(world: &mut CivWorld, _civ: CivArg) {
    world.submit(Command::Sentry {
        unit: world.settler_id(),
    });
}

#[when(expr = "the {civ} begin building {improvement}")]
async fn the_civ_begin_building(
    world: &mut CivWorld,
    _civ: CivArg,
    improvement: TerrainImprovementArg,
) {
    world.submit(Command::Work {
        unit: world.settler_id(),
        improvement: improvement.0,
    });
}

#[when(expr = "the {civ} cancel the settler's order")]
async fn the_civ_cancel(world: &mut CivWorld, _civ: CivArg) {
    world.submit(Command::CancelOrder {
        unit: world.settler_id(),
    });
}

#[when(expr = "the {civ} set {word} to produce a {word}")]
async fn the_civ_set_city_to_produce(
    world: &mut CivWorld,
    _civ: CivArg,
    city: String,
    unit: String,
) {
    let city_id = world.city();
    let founded_name = world
        .engine()
        .city(city_id)
        .expect("the founded city still exists")
        .name
        .clone();
    assert_eq!(
        founded_name, city,
        "the city being set to work is the founded city"
    );
    world.submit(Command::SetProductionTarget {
        city: city_id,
        target: ProductionTarget::Unit(unit_class(&unit)),
    });
}

#[then(expr = "the year is {year}")]
async fn the_year_is(world: &mut CivWorld, year: YearArg) {
    assert_eq!(world.engine().year(), year.0, "the calendar year");
}

#[then(expr = "it is turn {int}")]
async fn it_is_turn(world: &mut CivWorld, turn: u32) {
    assert_eq!(world.engine().turn(), turn, "the turn number");
}

#[then(expr = "the treasury holds {int} gold")]
async fn the_treasury_holds(world: &mut CivWorld, gold: u32) {
    assert_eq!(world.engine().gold(), gold, "the treasury balance");
}

#[then(expr = "the map is {int} tiles wide and {int} tiles tall")]
async fn the_map_is(world: &mut CivWorld, width: u32, height: u32) {
    assert_eq!(world.engine().width() as u32, width, "map width");
    assert_eq!(world.engine().height() as u32, height, "map height");
}

#[then(expr = "the {civ} starting tile is explored")]
async fn the_starting_tile_is_explored(world: &mut CivWorld, _civ: CivArg) {
    let origin = world.origin();
    assert!(
        world
            .engine()
            .explored(origin.x as usize, origin.y as usize),
        "the starting tile is revealed"
    );
}

#[then(expr = "the land around the {civ} starting tile is explored")]
async fn the_surroundings_are_explored(world: &mut CivWorld, _civ: CivArg) {
    let engine = world.engine();
    let origin = world.origin();
    for direction in [
        Direction::N,
        Direction::NE,
        Direction::E,
        Direction::SE,
        Direction::S,
        Direction::SW,
        Direction::W,
        Direction::NW,
    ] {
        if let Some(tile) = translate(engine.width(), engine.height(), origin, direction, 1) {
            assert!(
                engine.explored(tile.x as usize, tile.y as usize),
                "the tile {direction:?} of the starting tile is revealed"
            );
        }
    }
}

#[then(expr = "the land two tiles to the east of the {civ} starting tile is explored")]
async fn the_land_two_tiles_east_is_explored(world: &mut CivWorld, _civ: CivArg) {
    let tile = world.two_tiles_towards(Direction::E);
    assert!(
        world.engine().explored(tile.x as usize, tile.y as usize),
        "the tile two tiles east is revealed"
    );
}

#[then(expr = "the {civ} settler is one tile {dir} of where it started")]
async fn the_settler_is_one_tile(world: &mut CivWorld, _civ: CivArg, dir: DirArg) {
    assert_eq!(
        world.settler_location(),
        world.neighbour(dir.0),
        "the settler's position after moving"
    );
}

#[then(expr = "the {civ} settler stands where it started")]
async fn the_settler_stands_where_it_started(world: &mut CivWorld, _civ: CivArg) {
    assert_eq!(
        world.settler_location(),
        world.origin(),
        "a rejected move leaves the settler in place"
    );
}

#[then(expr = "the {civ} moving {dir} is reported")]
async fn the_moving_is_reported(world: &mut CivWorld, _civ: CivArg, dir: DirArg) {
    assert!(
        world.has_event(&format!(
            "Unit {} moves {:?}",
            world.settler_id().index(),
            dir.0
        )),
        "the move is announced"
    );
}

#[then(expr = "the {civ} settler's tile is {landscape}")]
async fn the_settler_s_tile_is(world: &mut CivWorld, _civ: CivArg, landscape: LandscapeArg) {
    let location = world.settler_location();
    let terrain = world
        .engine()
        .tile(location.x as usize, location.y as usize)
        .terrain;
    match landscape.0 {
        Landscape::OpenLand => {
            assert!(
                terrain.is_land() && terrain.movement_cost() == 1,
                "the tile is open land"
            );
        }
        Landscape::DenseTerrain => {
            assert!(
                terrain.is_land() && terrain.movement_cost() == 2,
                "the tile is dense ground"
            );
        }
        Landscape::OpenWater => assert!(terrain.is_water(), "the tile is open water"),
    }
}

#[then(expr = "the {civ} settler has no moves remaining")]
async fn the_settler_has_no_moves(world: &mut CivWorld, _civ: CivArg) {
    assert_eq!(
        world.settler().moves_remaining(),
        0,
        "every move point is spent"
    );
}

#[then(expr = "the {civ} report that the sea stops the settler")]
async fn the_sea_stops_the_settler(world: &mut CivWorld, _civ: CivArg) {
    assert!(
        world.has_event_containing("cannot cross land/sea border"),
        "the sea is impassable to a land unit"
    );
}

#[then(expr = "the {civ} report that the settler is spent")]
async fn the_settler_is_spent(world: &mut CivWorld, _civ: CivArg) {
    assert!(
        world.has_event_containing("has no moves left"),
        "a unit without moves cannot move"
    );
}

#[then(expr = "the city of {word} stands where the settler stood")]
async fn the_city_stands_where_the_settler_stood(world: &mut CivWorld, name: String) {
    let city = world.engine().city(world.city()).expect("the city exists");
    assert_eq!(city.name, name, "the founded city's name");
    assert_eq!(city.location, world.origin(), "the city's founding tile");
}

#[then(expr = "the {civ} have no settler left")]
async fn the_civ_have_no_settler(world: &mut CivWorld, _civ: CivArg) {
    assert!(
        world.engine().player_units().is_empty(),
        "the settler was consumed by founding"
    );
}

#[then(expr = "the {civ} beginning research on {advancement} is reported")]
async fn the_beginning_research_is_reported(
    world: &mut CivWorld,
    civ: CivArg,
    advancement: AdvancementArg,
) {
    assert!(
        world.has_event_containing(&format!(
            "{:?} begin researching {:?}",
            civ.0, advancement.0
        )),
        "the research start is announced"
    );
}

#[then(expr = "the {civ} are researching {advancement}")]
async fn the_civ_are_researching(world: &mut CivWorld, _civ: CivArg, advancement: AdvancementArg) {
    assert_eq!(
        world.engine().advancement_in_progress(),
        Some(advancement.0),
        "the advancement being studied"
    );
}

#[then(expr = "the {civ} are researching nothing")]
async fn the_civ_are_researching_nothing(world: &mut CivWorld, _civ: CivArg) {
    assert_eq!(
        world.engine().advancement_in_progress(),
        None,
        "no advancement is being studied"
    );
}

#[then(expr = "the {civ} research progress is {int}")]
async fn the_research_progress_is(world: &mut CivWorld, _civ: CivArg, progress: u32) {
    assert_eq!(
        world.engine().research_progress(),
        progress,
        "accumulated research"
    );
}

#[then(expr = "the {civ} have {advancement}")]
async fn the_civ_have_advancement(world: &mut CivWorld, civ: CivArg, advancement: AdvancementArg) {
    let player = world.player_id(civ.0);
    assert!(
        world
            .engine()
            .player_advances(player)
            .contains(&advancement.0),
        "the advancement has been discovered"
    );
}

#[then(expr = "the {civ} do not have {advancement}")]
async fn the_civ_do_not_have_advancement(
    world: &mut CivWorld,
    civ: CivArg,
    advancement: AdvancementArg,
) {
    let player = world.player_id(civ.0);
    assert!(
        !world
            .engine()
            .player_advances(player)
            .contains(&advancement.0),
        "the advancement has not been discovered"
    );
}

#[then(expr = "the {civ} discovering {advancement} is reported")]
async fn the_discovering_is_reported(
    world: &mut CivWorld,
    civ: CivArg,
    advancement: AdvancementArg,
) {
    assert!(
        world.has_event_containing(&format!("{:?} discover {:?}", civ.0, advancement.0)),
        "the discovery is announced"
    );
}

#[then(expr = "the city of {word} grows to size {int}")]
async fn the_city_grows_to_size(world: &mut CivWorld, name: String, size: u32) {
    let city = world.engine().city(world.city()).expect("the city exists");
    assert_eq!(city.name, name, "the growing city's name");
    assert_eq!(city.population(), size, "the city's population");
}

#[then(expr = "the {civ} report that the city of {word} grows")]
async fn the_growth_is_reported(world: &mut CivWorld, _civ: CivArg, name: String) {
    assert!(
        world.has_event_containing(&format!("{name} grows to size")),
        "the growth is announced"
    );
}

#[then(expr = "the city of {word} is still size {int}")]
async fn the_city_is_still_size(world: &mut CivWorld, name: String, size: u32) {
    let city = world.engine().city(world.city()).expect("the city exists");
    assert_eq!(city.name, name, "the city's name");
    assert_eq!(
        city.population(),
        size,
        "the city's population did not change"
    );
}

#[then(expr = "the {civ} report that the city of {word} is starving")]
async fn the_starvation_is_reported(world: &mut CivWorld, _civ: CivArg, name: String) {
    assert!(
        world.has_event_containing(&format!("{name} is starving")),
        "the starvation is announced"
    );
}

#[then(expr = "the city of {word} begins producing a {word} is reported")]
async fn the_beginning_production_is_reported(world: &mut CivWorld, _name: String, unit: String) {
    assert!(
        world.has_event_containing(&format!(
            "begins producing {:?}",
            ProductionTarget::Unit(unit_class(&unit))
        )),
        "the production start is announced"
    );
}

#[then(expr = "the city of {word} is producing a {word}")]
async fn the_city_is_producing(world: &mut CivWorld, name: String, unit: String) {
    let city = world.engine().city(world.city()).expect("the city exists");
    assert_eq!(city.name, name, "the city's name");
    assert_eq!(
        city.production_target(),
        Some(ProductionTarget::Unit(unit_class(&unit))),
        "the city's production target"
    );
}

#[then(expr = "the city of {word} producing a {word} is reported")]
async fn the_production_is_reported(world: &mut CivWorld, _name: String, unit: String) {
    assert!(
        world.has_event_containing(&format!("produces {:?}", unit_class(&unit))),
        "the completed unit is announced"
    );
}

#[then(expr = "the {civ} now have a {word} unit")]
async fn the_civ_now_have_a_unit(world: &mut CivWorld, _civ: CivArg, unit: String) {
    let class = unit_class(&unit);
    assert!(
        world
            .engine()
            .player_units()
            .into_iter()
            .any(|built| built.unit_class == class),
        "a freshly produced {unit} stands with the player's units"
    );
}

#[then(expr = "the {civ} do not have a {word} unit")]
async fn the_civ_do_not_have_a_unit(world: &mut CivWorld, _civ: CivArg, unit: String) {
    let class = unit_class(&unit);
    assert!(
        !world
            .engine()
            .player_units()
            .into_iter()
            .any(|built| built.unit_class == class),
        "no {unit} stands with the player's units"
    );
}

#[then(expr = "the city of {word} is owned by the {civ}")]
async fn the_city_is_owned_by(world: &mut CivWorld, name: String, civ: CivArg) {
    let city = world.engine().city(world.city()).expect("the city exists");
    assert_eq!(city.name, name, "the city's name");
    assert_eq!(
        world.city_owner(world.city()),
        civ.0,
        "the city's governing civilization"
    );
}

#[then(expr = "the {civ} declaring war on the {civ} is reported")]
async fn the_declaring_war_is_reported(world: &mut CivWorld, attacker: CivArg, defender: CivArg) {
    assert!(
        world.has_event_containing(&format!(
            "{:?} declares war on {:?}",
            attacker.0, defender.0
        )),
        "the declaration of war is announced"
    );
}

#[then(expr = "the {civ} report that they are already at war with the {civ}")]
async fn the_already_at_war_is_reported(
    world: &mut CivWorld,
    _attacker: CivArg,
    _defender: CivArg,
) {
    assert!(
        world.has_event_containing("Already at war"),
        "a second declaration is refused"
    );
}

#[then(expr = "the {civ} making peace with the {civ} is reported")]
async fn the_making_peace_is_reported(world: &mut CivWorld, attacker: CivArg, defender: CivArg) {
    assert!(
        world.has_event_containing(&format!(
            "{:?} makes peace with {:?}",
            attacker.0, defender.0
        )),
        "the peace treaty is announced"
    );
}

#[then(expr = "the {civ} report that they are already at peace with the {civ}")]
async fn the_already_at_peace_is_reported(
    world: &mut CivWorld,
    _attacker: CivArg,
    _defender: CivArg,
) {
    assert!(
        world.has_event_containing("Already at peace"),
        "a second treaty is refused"
    );
}

#[then(expr = "the {civ} declaring war on themselves is refused")]
async fn the_war_on_self_is_refused(world: &mut CivWorld, _civ: CivArg) {
    assert!(
        world.has_event_containing("Cannot declare war on yourself"),
        "war with yourself is rejected"
    );
}

#[then(expr = "the {civ} making peace with themselves is refused")]
async fn the_peace_with_self_is_refused(world: &mut CivWorld, _civ: CivArg) {
    assert!(
        world.has_event_containing("Cannot make peace with yourself"),
        "peace with yourself is rejected"
    );
}

#[then(expr = "the {civ} settler is fortified")]
async fn the_settler_is_fortified(world: &mut CivWorld, _civ: CivArg) {
    assert_eq!(
        world.settler().order(),
        UnitOrder::Fortified,
        "the settler's order"
    );
}

#[then(expr = "the {civ} settler is on sentry")]
async fn the_settler_is_on_sentry(world: &mut CivWorld, _civ: CivArg) {
    assert_eq!(
        world.settler().order(),
        UnitOrder::Sentried,
        "the settler's order"
    );
}

#[then(expr = "the {civ} settler beginning work on {improvement} is reported")]
async fn the_beginning_work_is_reported(
    world: &mut CivWorld,
    _civ: CivArg,
    improvement: TerrainImprovementArg,
) {
    assert!(
        world.has_event_containing(&format!("begins building {:?}", improvement.0)),
        "the start of construction is announced"
    );
}

#[then(expr = "the {civ} settler finishing {improvement} is reported")]
async fn the_finishing_work_is_reported(
    world: &mut CivWorld,
    _civ: CivArg,
    improvement: TerrainImprovementArg,
) {
    assert!(
        world.has_event_containing(&format!("finishes {:?}", improvement.0)),
        "the completion of a built improvement is announced"
    );
}

#[then(expr = "the tile the {civ} settler stands on is {improved}")]
async fn the_settler_s_tile_is_improved(
    world: &mut CivWorld,
    _civ: CivArg,
    improved: TerrainStateArg,
) {
    assert!(
        tile_has_improvement(world, improved.0),
        "the tile carries the improvement"
    );
}

#[then(expr = "the tile the {civ} settler stands on is not {improved}")]
async fn the_settler_s_tile_is_not_improved(
    world: &mut CivWorld,
    _civ: CivArg,
    improved: TerrainStateArg,
) {
    assert!(
        !tile_has_improvement(world, improved.0),
        "the tile does not carry the improvement"
    );
}

#[then(expr = "the {civ} settler's order is cancelled")]
async fn the_settler_s_order_is_cancelled(world: &mut CivWorld, _civ: CivArg) {
    assert_eq!(
        world.settler().order(),
        UnitOrder::Idle,
        "the settler's order"
    );
    assert!(
        world.has_event_containing("order cancelled"),
        "the cancellation is announced"
    );
}

/// Whether the tile under the settler carries `improvement`.
fn tile_has_improvement(world: &CivWorld, improvement: TerrainImprovement) -> bool {
    let location = world.settler_location();
    let tile = world
        .engine()
        .tile(location.x as usize, location.y as usize);
    match improvement {
        TerrainImprovement::Road => tile.has_road(),
        TerrainImprovement::Mine => tile.is_mined(),
        TerrainImprovement::Irrigation => tile.is_irrigated(),
    }
}
