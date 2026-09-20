use super::*;
use crate::game_engine::calendar::calendar_year;
use crate::game_engine::{Command, GameView, Player};
use crate::model::advancements::Advancement;
use crate::model::cartography::Direction;
use crate::model::cartography::Location;
use crate::model::cities::{City, CityId, CityImprovement, ProductionTarget};
use crate::model::civilizations::Civilization;
use crate::model::civilizations::PlayerId;
use crate::model::geography::Terrain;
use crate::model::geography::TerrainImprovement;
use crate::model::units::{UnitClass, UnitId, UnitOrder};

fn english_player() -> Player {
    Player::new(Civilization::English)
}

#[test]
fn the_default_map_is_80_by_50() {
    let engine = Engine::default();
    assert_eq!(engine.width(), DEFAULT_MAP_WIDTH);
    assert_eq!(engine.height(), DEFAULT_MAP_HEIGHT);
    assert_eq!(DEFAULT_MAP_WIDTH, 80);
    assert_eq!(DEFAULT_MAP_HEIGHT, 50);
}

#[test]
fn the_calendar_starts_in_4000_bc_at_50_years_a_turn() {
    assert_eq!(calendar_year(1), -4000);
    assert_eq!(calendar_year(2), -3950);
    assert_eq!(calendar_year(10), -3550);
}

#[test]
fn the_calendar_speeds_up_through_the_eras() {
    // 50 years per turn until 1000 BC.
    assert_eq!(calendar_year(61), -1000);
    // 25 years per turn from 1000 BC, ending in 1 BC (astronomical 0).
    assert_eq!(calendar_year(62), -975);
    assert_eq!(calendar_year(100), -25);
    // No year 0: the calendar jumps straight from 25 BC to 1 AD.
    assert_eq!(calendar_year(101), 0);
    // 10 years per turn across the early AD era.
    assert_eq!(calendar_year(102), 10);
    assert_eq!(calendar_year(151), 500);
    // 5 years per turn through the middle ages.
    assert_eq!(calendar_year(152), 505);
    assert_eq!(calendar_year(351), 1500);
}

#[test]
fn the_calendar_keeps_advancing_after_the_schedule_ends() {
    // 825 calendar steps take the game from 4000 BC to 2100 AD on turn 826.
    assert_eq!(calendar_year(726), 2000);
    assert_eq!(calendar_year(826), 2100);
    assert_eq!(calendar_year(827), 2101);
}

#[test]
fn the_engine_reports_the_first_turn_as_4000_bc() {
    let engine = Engine::default();
    assert_eq!(engine.year(), -4000);
}

fn test_engine() -> Engine {
    let mut engine = Engine::new(3, 2, english_player(), Vec::new());
    engine.game.spawn_unit(
        UnitClass::Settler,
        Location::new(1, 1),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine
}

#[test]
fn populate_starting_world_gives_every_player_a_settler_on_land_with_reveals() {
    let mut engine = Engine::new(
        12,
        10,
        Player::new(Civilization::English),
        vec![Player::new(Civilization::Zulu)],
    );
    engine.populate_starting_world();

    for index in 0..engine.game.players.len() {
        let owner = PlayerId::new(index);
        let settler = engine
            .game
            .units
            .iter()
            .find(|unit| unit.owner() == owner && unit.unit_class == UnitClass::Settler)
            .expect("every player starts with a settler");
        let location = settler.location;
        assert!(
            engine.game.map.tile_at(location).terrain.is_land(),
            "settler must sit on land at {location:?}"
        );
        assert!(
            engine.game.players[index].explored_at(location.x as usize, location.y as usize),
            "the settler's tile is revealed"
        );
        assert!(
            engine.game.players[index].explored_at(location.x as usize + 1, location.y as usize),
            "the tile beside the settler is revealed"
        );
    }
}

#[test]
fn populate_starting_world_places_each_settler_on_a_distinct_tile() {
    let mut engine = Engine::new(
        12,
        10,
        Player::new(Civilization::English),
        (0..4)
            .map(|i| {
                Player::new(match i {
                    0 => Civilization::Roman,
                    1 => Civilization::Greek,
                    2 => Civilization::Zulu,
                    _ => Civilization::American,
                })
            })
            .collect(),
    );
    engine.populate_starting_world();
    let locations: Vec<(u16, u16)> = engine
        .game
        .units
        .iter()
        .map(|unit| (unit.location.x, unit.location.y))
        .collect();
    let unique: std::collections::HashSet<(u16, u16)> = locations.iter().copied().collect();
    assert_eq!(
        unique.len(),
        locations.len(),
        "each settler lands on its own tile"
    );
}

#[test]
fn no_settler_starts_on_tundra() {
    let mut engine = Engine::new(
        12,
        10,
        Player::new(Civilization::English),
        vec![Player::new(Civilization::Zulu)],
    );
    engine.populate_starting_world();
    for unit in engine.game.units.iter() {
        if unit.unit_class == UnitClass::Settler {
            assert_ne!(
                engine.game.map.tile_at(unit.location).terrain,
                Terrain::Tundra,
                "a settler was placed on tundra at {:?}",
                unit.location
            );
        }
    }
}

#[test]
fn different_seeds_produce_different_worlds() {
    let signature = |seed: u64| {
        let mut engine =
            Engine::with_seed(24, 16, Player::new(Civilization::English), Vec::new(), seed);
        engine.populate_starting_world();
        let mut terrain = Vec::new();
        for y in 0..engine.height() {
            for x in 0..engine.width() {
                terrain.push(engine.tile(x, y).terrain.as_char());
            }
        }
        terrain
    };
    assert_ne!(
        signature(7),
        signature(8),
        "different seeds should generate different terrain"
    );
}

fn two_player_engine() -> Engine {
    let mut engine = Engine::new(
        3,
        2,
        Player::new(Civilization::English),
        vec![Player::new(Civilization::Zulu)],
    );
    engine.game.spawn_unit(
        UnitClass::Settler,
        Location::new(1, 1),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine
}

fn three_player_engine() -> Engine {
    Engine::new(
        3,
        2,
        Player::new(Civilization::English),
        vec![
            Player::new(Civilization::Zulu),
            Player::new(Civilization::Roman),
        ],
    )
}

#[test]
fn engine_starts_playing_the_first_player() {
    let engine = test_engine();
    assert_eq!(engine.current_player(), Civilization::English);
    assert_eq!(engine.turn(), 1);
}

#[test]
fn view_exposes_map_dimensions() {
    let engine = test_engine();
    assert_eq!(engine.width(), 3);
    assert_eq!(engine.height(), 2);
}

#[test]
fn view_reports_tiles_and_units_by_location() {
    let engine = test_engine();
    assert_eq!(engine.tile(0, 0).terrain, Terrain::Ocean);
    assert_eq!(engine.units_at(1, 1).len(), 1);
    assert!(engine.units_at(0, 0).is_empty());
}

#[test]
fn move_command_moves_the_unit_within_the_map() {
    let mut engine = test_engine();
    engine.game.map.tile_at_mut(Location::new(2, 1)).terrain = Terrain::Grassland;
    let events = engine.submit(Command::Move {
        unit: UnitId::new(0),
        direction: Direction::E,
    });
    assert_eq!(engine.game.units[0].location, Location::new(2, 1));
    assert_eq!(events[0].message(), "Unit 0 moves E");
}

#[test]
fn move_command_reports_an_event_when_the_target_is_off_the_map() {
    let mut engine = test_engine();
    let events = engine.submit(Command::Move {
        unit: UnitId::new(0),
        direction: Direction::S,
    });
    assert_eq!(events[0].message(), "Cannot move there");
    assert_eq!(engine.game.units[0].location, Location::new(1, 1));
}

#[test]
fn moving_east_off_the_map_wraps_around_to_the_west() {
    let mut engine = test_engine();
    engine.game.map.tile_at_mut(Location::new(2, 1)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(0, 1)).terrain = Terrain::Grassland;
    engine.submit(Command::Move {
        unit: UnitId::new(0),
        direction: Direction::E,
    });
    engine.submit(Command::EndTurn);
    engine.submit(Command::Move {
        unit: UnitId::new(0),
        direction: Direction::E,
    });
    assert_eq!(engine.game.units[0].location, Location::new(0, 1));
}

#[test]
fn moving_west_off_the_map_wraps_around_to_the_east() {
    let mut engine = test_engine();
    engine.game.map.tile_at_mut(Location::new(0, 1)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(2, 1)).terrain = Terrain::Grassland;
    engine.submit(Command::Move {
        unit: UnitId::new(0),
        direction: Direction::W,
    });
    engine.submit(Command::EndTurn);
    engine.submit(Command::Move {
        unit: UnitId::new(0),
        direction: Direction::W,
    });
    assert_eq!(engine.game.units[0].location, Location::new(2, 1));
}

#[test]
fn move_command_reports_an_event_for_an_unknown_unit() {
    let mut engine = test_engine();
    let events = engine.submit(Command::Move {
        unit: UnitId::new(99),
        direction: Direction::N,
    });
    assert_eq!(events[0].message(), "No such unit");
}

#[test]
fn commanding_another_players_unit_is_rejected() {
    let mut engine = two_player_engine();
    let legion = engine.game.spawn_unit(
        UnitClass::Legion,
        Location::new(2, 0),
        PlayerId::new(1),
        CityId::new(0),
    );
    let events = engine.submit(Command::Fortify { unit: legion });
    assert_eq!(events[0].message(), "No such unit");
    assert_eq!(
        engine
            .game
            .units
            .iter()
            .find(|u| u.id() == legion)
            .unwrap()
            .order(),
        UnitOrder::Idle
    );
}

#[test]
fn fortify_command_orders_the_unit_and_reports_an_event() {
    let mut engine = test_engine();
    let events = engine.submit(Command::Fortify {
        unit: UnitId::new(0),
    });
    assert_eq!(engine.game.units[0].order(), UnitOrder::Fortified);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].message(), "Unit 0 fortifies");
}

#[test]
fn sentry_command_orders_the_unit() {
    let mut engine = test_engine();
    engine.submit(Command::Sentry {
        unit: UnitId::new(0),
    });
    assert_eq!(engine.game.units[0].order(), UnitOrder::Sentried);
}

#[test]
fn work_command_orders_the_unit_but_the_improvement_lands_later() {
    let mut engine = test_engine();
    engine.game.map.tile_at_mut(Location::new(1, 1)).terrain = Terrain::Grassland;
    assert!(!engine.game.map.tile_at(Location::new(1, 1)).has_road());
    let events = engine.submit(Command::Work {
        unit: UnitId::new(0),
        improvement: TerrainImprovement::Road,
    });
    // The settler is ordered to build, but the tile stays untouched until
    // the build finishes.
    assert!(!engine.game.map.tile_at(Location::new(1, 1)).has_road());
    assert_eq!(
        engine.game.units[0].order(),
        UnitOrder::Improving(TerrainImprovement::Road)
    );
    assert_eq!(engine.game.units[0].work_progress(), 1);
    assert_eq!(engine.game.units[0].moves_remaining(), 0);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].message(), "Unit 0 begins building Road");
}

#[test]
fn a_road_build_takes_two_turns() {
    let mut engine = test_engine();
    engine.game.map.tile_at_mut(Location::new(1, 1)).terrain = Terrain::Grassland;
    engine.submit(Command::Work {
        unit: UnitId::new(0),
        improvement: TerrainImprovement::Road,
    });
    let events = engine.submit(Command::EndTurn);
    assert!(
        engine.game.map.tile_at(Location::new(1, 1)).has_road(),
        "the road lands on the settler's second turn of work"
    );
    assert_eq!(engine.game.units[0].order(), UnitOrder::Idle);
    assert!(events.iter().any(|e| e.message() == "Unit 0 finishes Road"));
}

#[test]
fn an_irrigation_build_takes_two_turns() {
    let mut engine = test_engine();
    engine.game.map.tile_at_mut(Location::new(1, 1)).terrain = Terrain::Grassland;
    engine.submit(Command::Work {
        unit: UnitId::new(0),
        improvement: TerrainImprovement::Irrigation,
    });
    assert!(!engine.game.map.tile_at(Location::new(1, 1)).is_irrigated());
    engine.submit(Command::EndTurn);
    assert!(engine.game.map.tile_at(Location::new(1, 1)).is_irrigated());
    assert_eq!(engine.game.units[0].order(), UnitOrder::Idle);
}

#[test]
fn a_mine_build_takes_three_turns() {
    let mut engine = test_engine();
    engine.game.map.tile_at_mut(Location::new(1, 1)).terrain = Terrain::Mountain;
    engine.submit(Command::Work {
        unit: UnitId::new(0),
        improvement: TerrainImprovement::Mine,
    });

    // One more turn of work: still mid-build, no moves back, no output.
    engine.submit(Command::EndTurn);
    assert!(!engine.game.map.tile_at(Location::new(1, 1)).is_mined());
    assert_eq!(
        engine.game.units[0].order(),
        UnitOrder::Improving(TerrainImprovement::Mine)
    );
    assert_eq!(engine.game.units[0].work_progress(), 2);
    assert_eq!(engine.game.units[0].moves_remaining(), 0);

    // The third turn finishes the mine.
    let events = engine.submit(Command::EndTurn);
    assert!(engine.game.map.tile_at(Location::new(1, 1)).is_mined());
    assert_eq!(engine.game.units[0].order(), UnitOrder::Idle);
    assert!(events.iter().any(|e| e.message() == "Unit 0 finishes Mine"));
}

#[test]
fn cancel_order_aborts_a_build_before_it_finishes() {
    let mut engine = test_engine();
    engine.game.map.tile_at_mut(Location::new(1, 1)).terrain = Terrain::Grassland;
    engine.submit(Command::Work {
        unit: UnitId::new(0),
        improvement: TerrainImprovement::Road,
    });
    engine.submit(Command::CancelOrder {
        unit: UnitId::new(0),
    });
    assert_eq!(engine.game.units[0].order(), UnitOrder::Idle);
    assert_eq!(engine.game.units[0].work_progress(), 0);
    engine.submit(Command::EndTurn);
    assert!(
        !engine.game.map.tile_at(Location::new(1, 1)).has_road(),
        "a cancelled build never lands"
    );
}

#[test]
fn work_command_rejects_unsupported_improvement() {
    let mut engine = test_engine();
    engine.game.map.tile_at_mut(Location::new(1, 1)).terrain = Terrain::Grassland;
    let events = engine.submit(Command::Work {
        unit: UnitId::new(0),
        improvement: TerrainImprovement::Mine,
    });
    assert!(!engine.game.map.tile_at(Location::new(1, 1)).is_mined());
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].message(), "Cannot build Mine here");
}

#[test]
fn work_command_is_limited_to_settlers() {
    let mut engine = test_engine();
    let location = Location::new(1, 0);
    engine.game.map.tile_at_mut(location).terrain = Terrain::Grassland;
    let legion = engine.game.spawn_unit(
        UnitClass::Legion,
        location,
        PlayerId::new(0),
        CityId::new(0),
    );
    let events = engine.submit(Command::Work {
        unit: legion,
        improvement: TerrainImprovement::Road,
    });
    assert!(!engine.game.map.tile_at(location).has_road());
    assert_eq!(engine.game.units.last().unwrap().order(), UnitOrder::Idle);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].message(), "Only settlers can build improvements");
}

#[test]
fn cancel_order_command_resets_the_unit() {
    let mut engine = test_engine();
    engine.submit(Command::Fortify {
        unit: UnitId::new(0),
    });
    engine.submit(Command::CancelOrder {
        unit: UnitId::new(0),
    });
    assert_eq!(engine.game.units[0].order(), UnitOrder::Idle);
}

#[test]
fn an_order_consumes_the_units_turn() {
    let mut engine = test_engine();
    let cavalry = engine.game.spawn_unit(
        UnitClass::Cavalry,
        Location::new(0, 0),
        PlayerId::new(0),
        CityId::new(0),
    );
    assert_eq!(engine.game.units[1].moves_remaining(), 3);
    engine.submit(Command::Sentry { unit: cavalry });
    assert_eq!(engine.game.units[1].moves_remaining(), 0);
}

#[test]
fn cancelling_an_order_also_consumes_the_turn() {
    let mut engine = test_engine();
    engine.submit(Command::Fortify {
        unit: UnitId::new(0),
    });
    engine.submit(Command::CancelOrder {
        unit: UnitId::new(0),
    });
    assert_eq!(engine.game.units[0].order(), UnitOrder::Idle);
    assert_eq!(engine.game.units[0].moves_remaining(), 0);
}

#[test]
fn unfortify_rallies_a_rested_garrison_without_spending_its_turn() {
    let mut engine = test_engine();
    engine.submit(Command::Fortify {
        unit: UnitId::new(0),
    });
    assert_eq!(engine.game.units[0].moves_remaining(), 0);
    // The next turn restores the garrison's moves but keeps it fortified.
    engine.begin_turn();
    let moves = engine.game.units[0].moves_remaining();
    assert!(moves > 0);
    assert_eq!(engine.game.units[0].order(), UnitOrder::Fortified);
    // Unfortifying clears the order without spending any of that budget, so
    // the unit is back in the available-units loop exactly as it stands.
    let events = engine.submit(Command::Unfortify {
        unit: UnitId::new(0),
    });
    assert_eq!(engine.game.units[0].order(), UnitOrder::Idle);
    assert_eq!(engine.game.units[0].moves_remaining(), moves);
    assert_eq!(events[0].message(), "Unit 0 is no longer fortified");
}

#[test]
fn unfortify_leaves_any_other_order_alone() {
    let mut engine = test_engine();
    let cavalry = engine.game.spawn_unit(
        UnitClass::Cavalry,
        Location::new(0, 0),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine.submit(Command::Sentry { unit: cavalry });
    assert_eq!(engine.game.units[1].order(), UnitOrder::Sentried);
    let events = engine.submit(Command::Unfortify { unit: cavalry });
    assert_eq!(
        events[0].message(),
        "Unit 1 is not fortified",
        "unfortify must not cancel a sentry order"
    );
    assert_eq!(engine.game.units[1].order(), UnitOrder::Sentried);
}

#[test]
fn submitting_to_an_unknown_unit_reports_an_event_without_changing_state() {
    let mut engine = test_engine();
    let events = engine.submit(Command::Fortify {
        unit: UnitId::new(99),
    });
    assert_eq!(events[0].message(), "No such unit");
    assert_eq!(engine.game.units[0].order(), UnitOrder::Idle);
}

#[test]
fn moving_a_unit_reveals_tiles_around_its_new_location() {
    let mut engine = Engine::new(5, 5, Player::new(Civilization::English), Vec::new());
    engine.game.spawn_unit(
        UnitClass::Settler,
        Location::new(0, 0),
        PlayerId::new(0),
        CityId::new(0),
    );
    assert!(engine.explored(1, 1));
    assert!(!engine.explored(4, 4));
    engine.game.map.tile_at_mut(Location::new(1, 0)).terrain = Terrain::Grassland;
    engine.submit(Command::Move {
        unit: UnitId::new(0),
        direction: Direction::E,
    });
    assert!(engine.explored(2, 1));
    assert!(!engine.explored(4, 4));
}

#[test]
fn moving_only_reveals_for_the_units_owner() {
    let mut engine = two_player_engine();
    engine.game.spawn_unit(
        UnitClass::Settler,
        Location::new(1, 0),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine.game.map.tile_at_mut(Location::new(2, 0)).terrain = Terrain::Grassland;
    engine.submit(Command::Move {
        unit: UnitId::new(1),
        direction: Direction::E,
    });
    assert!(engine.game.players[0].explored_at(2, 0));
    assert!(!engine.game.players[1].explored_at(2, 0));
}

#[test]
fn end_turn_advances_the_turn_number() {
    let mut engine = test_engine();
    let events = engine.submit(Command::EndTurn);
    assert_eq!(engine.turn(), 2);
    assert_eq!(events[0].message(), "English begins turn 2");
}

#[test]
fn end_turn_resolves_the_whole_round_back_to_the_human() {
    let mut engine = two_player_engine();
    let events = engine.submit(Command::EndTurn);
    assert_eq!(engine.current_player(), Civilization::English);
    assert_eq!(engine.turn(), 2);
    assert_eq!(events[0].message(), "Zulu begins turn 1");
    assert_eq!(events.last().unwrap().message(), "English begins turn 2");
}

#[test]
fn a_rivals_garrison_march_is_recorded_for_the_replay_and_drains_once() {
    // A wide grassland world where the rival's legion must close on its city
    // to garrison it: that march is rival motion the TUI replays, and the
    // human's own moves are never recorded.
    let mut engine = Engine::new(
        15,
        3,
        Player::new(Civilization::English),
        vec![Player::new(Civilization::Zulu)],
    );
    for y in 0..3 {
        for x in 0..15 {
            engine
                .game
                .map
                .tile_at_mut(Location::new(x as u16, y as u16))
                .terrain = Terrain::Grassland;
        }
    }
    engine.game.spawn_unit(
        UnitClass::Settler,
        Location::new(1, 0),
        PlayerId::new(0),
        CityId::new(0),
    );
    let legion = engine.game.spawn_unit(
        UnitClass::Legion,
        Location::new(4, 1),
        PlayerId::new(1),
        CityId::new(0),
    );
    engine.game.cities.push(City::new(
        "Ulundi",
        Location::new(14, 1),
        PlayerId::new(1),
        CityId::new(1),
    ));
    // The human's settler steps east of its own accord; its move must not be
    // recorded as motion.
    engine.submit(Command::Move {
        unit: UnitId::new(0),
        direction: Direction::E,
    });

    engine.submit(Command::EndTurn);
    assert_eq!(engine.current_player(), Civilization::English);
    assert_eq!(engine.turn(), 2);

    let motion: Vec<RivalMotion> = engine.drain_rival_motion();
    assert!(
        !motion.is_empty(),
        "the legion's march toward its city is replayed"
    );
    assert!(
        motion.iter().all(|step| step.unit == legion),
        "every recorded step belongs to the rival's legion, never the human's \
         settler"
    );
    for pair in motion.windows(2) {
        assert_eq!(pair[0].to, pair[1].from, "the steps chain tile to tile");
    }
    let stationed = engine
        .game
        .units
        .iter()
        .find(|unit| unit.id() == legion)
        .expect("the legion still exists");
    assert_eq!(
        motion.last().unwrap().to,
        stationed.location,
        "the final step ends where the unit now stands"
    );
    assert!(
        engine.drain_rival_motion().is_empty(),
        "draining the record empties it until the next round"
    );
}

/// Zulu cities planted at `left` and `right`, their settler already standing
/// on the terrain the AI will pick, and the whole map explored. Returns the
/// engine with the human passed for the turn.
fn rival_settlement_engine(left: Location, right: Location, settler: Location) -> Engine {
    let mut engine = Engine::new(
        15,
        3,
        Player::new(Civilization::English),
        vec![Player::new(Civilization::Zulu)],
    );
    for y in 0..3 {
        for x in 0..15 {
            engine
                .game
                .map
                .tile_at_mut(Location::new(x as u16, y as u16))
                .terrain = Terrain::Grassland;
        }
    }
    engine.game.spawn_unit(
        UnitClass::Settler,
        Location::new(14, 0),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine.game.spawn_unit(
        UnitClass::Settler,
        settler,
        PlayerId::new(1),
        CityId::new(0),
    );
    engine
        .game
        .cities
        .push(City::new("Angkor", left, PlayerId::new(1), CityId::new(1)));
    engine
        .game
        .cities
        .push(City::new("Annam", right, PlayerId::new(1), CityId::new(2)));
    engine.game.players[1].reveal_tiles_at(Location::new(7, 1), 8);
    engine
}

/// How many of `new_city`'s 21 working tiles sit inside the union of every
/// other Zulu city's working grid.
fn footprint_overlap(engine: &Engine, new_city: Location) -> usize {
    let existing: Vec<Location> = engine
        .game
        .cities
        .iter()
        .filter(|city| city.owner() == PlayerId::new(1) && city.location != new_city)
        .flat_map(|city| engine.game.city_footprint(city.location))
        .collect();
    engine
        .game
        .city_footprint(new_city)
        .iter()
        .filter(|tile| existing.contains(tile))
        .count()
}

#[test]
fn a_rival_settler_will_not_crowd_an_existing_city() {
    // Cities at columns 2 and 12 leave columns 6-8 untouched; the settler
    // stands on (6,0), the leftmost of those. Long-explored nearer tiles
    // like (0,0) sit well inside Angkor's working grid and must lose out.
    let mut engine = rival_settlement_engine(
        Location::new(2, 1),
        Location::new(12, 1),
        Location::new(6, 0),
    );

    engine.submit(Command::EndTurn);

    let zulu_cities: Vec<Location> = engine
        .game
        .cities
        .iter()
        .filter(|city| city.owner() == PlayerId::new(1))
        .map(|city| city.location)
        .collect();
    assert_eq!(zulu_cities.len(), 3, "the settler founds its city");
    assert_eq!(
        zulu_cities
            .iter()
            .find(|location| **location != Location::new(2, 1)
                && **location != Location::new(12, 1))
            .copied(),
        Some(Location::new(6, 0)),
        "the open ground is chosen over the crowded tiles nearer Angkor"
    );
    let shared = footprint_overlap(&engine, Location::new(6, 0));
    assert!(
        shared <= 3,
        "a new city shares at most 3 working tiles with its neighbours, \
         shared {shared}"
    );
    assert!(
        shared > 0,
        "the chosen site sits up against the old grid's edge, not inside it"
    );
    assert_eq!(engine.current_player(), Civilization::English);
}

#[test]
fn a_rival_settler_still_founds_when_only_crowded_sites_remain() {
    // Cities at columns 3 and 10 put every column within Chebyshev 3 of one
    // of them, so no explored land site can obey the working-grid law: the
    // fallback must fire and the settler must still found, on the best
    // crowded tile (0,0), rather than stall.
    let mut engine = rival_settlement_engine(
        Location::new(3, 1),
        Location::new(10, 1),
        Location::new(0, 0),
    );
    for y in 0..3 {
        for x in 0..15 {
            let location = Location::new(x as u16, y as u16);
            if ![(3, 1), (10, 1)].contains(&(x, y)) {
                assert!(
                    footprint_overlap(&engine, location) > 3,
                    "the scenario leaves no spacious site at {location:?}"
                );
            }
        }
    }

    engine.submit(Command::EndTurn);

    let zulu_cities: Vec<Location> = engine
        .game
        .cities
        .iter()
        .filter(|city| city.owner() == PlayerId::new(1))
        .map(|city| city.location)
        .collect();
    assert_eq!(zulu_cities.len(), 3, "the rival founds even on a full map");
    assert!(
        zulu_cities.contains(&Location::new(0, 0)),
        "the best crowded tile wins once no open one exists"
    );
}

/// A 15x3 grassland world holding nothing but the Zulu capital at `capital`
/// and its settler at `settler`, with nothing revealed beyond the capital's own
/// footprint — the exact state right after a city is founded and its first
/// settler is produced, when every visible tile still counts as crowded.
fn frontier_settlement_engine(capital: Location, settler: Location) -> Engine {
    let mut engine = Engine::new(
        15,
        3,
        Player::new(Civilization::English),
        vec![Player::new(Civilization::Zulu)],
    );
    for y in 0..3 {
        for x in 0..15 {
            engine
                .game
                .map
                .tile_at_mut(Location::new(x as u16, y as u16))
                .terrain = Terrain::Grassland;
        }
    }
    engine.game.spawn_unit(
        UnitClass::Settler,
        Location::new(14, 0),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine.game.cities.push(City::new(
        "Angkor",
        capital,
        PlayerId::new(1),
        CityId::new(1),
    ));
    engine.game.spawn_unit(
        UnitClass::Settler,
        settler,
        PlayerId::new(1),
        CityId::new(1),
    );
    engine.game.players[1].reveal_tiles_surrounding_city_at(capital);
    engine
}

/// How many of `new_city`'s 21 working tiles sit inside `other` city's working
/// grid, whatever their owners.
fn footprint_overlap_with(engine: &Engine, new_city: Location, other: Location) -> usize {
    engine
        .game
        .city_footprint(other)
        .iter()
        .filter(|tile| engine.game.city_footprint(new_city).contains(tile))
        .count()
}

#[test]
fn a_rival_settler_seeks_open_ground_before_founding_in_the_crowd() {
    // The capital's footprint is all the settler can see: every visible tile
    // sits within Chebyshev 2 of the capital, so found-in-place is living in
    // the crowd. The settler must instead push the frontier and only found
    // once its marches open up genuinely open ground.
    let mut engine = frontier_settlement_engine(Location::new(10, 1), Location::new(10, 1));

    engine.submit(Command::EndTurn);
    let zulu_cities: Vec<Location> = engine
        .game
        .cities
        .iter()
        .filter(|city| city.owner() == PlayerId::new(1))
        .map(|city| city.location)
        .collect();
    assert_eq!(
        zulu_cities.len(),
        1,
        "no cramped founding on the first turn"
    );

    for _ in 0..8 {
        engine.submit(Command::EndTurn);
        if engine
            .game
            .cities
            .iter()
            .filter(|city| city.owner() == PlayerId::new(1))
            .count()
            >= 2
        {
            break;
        }
    }
    let zulu_cities: Vec<Location> = engine
        .game
        .cities
        .iter()
        .filter(|city| city.owner() == PlayerId::new(1))
        .map(|city| city.location)
        .collect();
    assert_eq!(zulu_cities.len(), 2, "the settler finds a second city");
    let new_city = zulu_cities
        .iter()
        .copied()
        .find(|location| *location != Location::new(10, 1))
        .expect("the new city is not the capital");
    let shared = footprint_overlap(&engine, new_city);
    assert!(
        shared <= 3,
        "the new city lands on open ground (shared {shared}), not in the \
         capital's lap"
    );
    assert_eq!(engine.current_player(), Civilization::English);
}

#[test]
fn a_rival_settler_will_not_crowd_a_human_city() {
    // Two English cities flank the Zulu settler exactly as the rival's own
    // cities did in `a_rival_settler_will_not_crowd_an_existing_city`. The
    // working-grid law now holds against every civilization, so the capital
    // still lands clear at (6,0) rather than inside a foreign grid.
    let mut engine = Engine::new(
        15,
        3,
        Player::new(Civilization::English),
        vec![Player::new(Civilization::Zulu)],
    );
    for y in 0..3 {
        for x in 0..15 {
            engine
                .game
                .map
                .tile_at_mut(Location::new(x as u16, y as u16))
                .terrain = Terrain::Grassland;
        }
    }
    engine.game.spawn_unit(
        UnitClass::Settler,
        Location::new(14, 0),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine.game.spawn_unit(
        UnitClass::Settler,
        Location::new(6, 0),
        PlayerId::new(1),
        CityId::new(0),
    );
    engine.game.cities.push(City::new(
        "Londinium",
        Location::new(2, 1),
        PlayerId::new(0),
        CityId::new(1),
    ));
    engine.game.cities.push(City::new(
        "Eboracum",
        Location::new(12, 1),
        PlayerId::new(0),
        CityId::new(2),
    ));
    engine.game.players[1].reveal_tiles_at(Location::new(7, 1), 8);

    engine.submit(Command::EndTurn);

    let zulu_cities: Vec<Location> = engine
        .game
        .cities
        .iter()
        .filter(|city| city.owner() == PlayerId::new(1))
        .map(|city| city.location)
        .collect();
    assert_eq!(zulu_cities.len(), 1, "the settler founds its capital");
    assert_eq!(
        zulu_cities[0],
        Location::new(6, 0),
        "the open ground is chosen over the tiles overlapping an English grid"
    );
    for occupied in [Location::new(2, 1), Location::new(12, 1)] {
        let shared = footprint_overlap_with(&engine, Location::new(6, 0), occupied);
        assert!(
            shared <= 3,
            "the rival keeps clear of the English grid (shared {shared})"
        );
    }
    assert_eq!(engine.current_player(), Civilization::English);
}

#[test]
fn a_rival_settler_will_not_crowd_another_rivals_city() {
    // Same geometry, but the guarding city belongs to a second rival (the
    // Aztecs): one rival's planning must respect another rival's grid too.
    let mut engine = Engine::new(
        15,
        3,
        Player::new(Civilization::English),
        vec![
            Player::new(Civilization::Zulu),
            Player::new(Civilization::Aztec),
        ],
    );
    for y in 0..3 {
        for x in 0..15 {
            engine
                .game
                .map
                .tile_at_mut(Location::new(x as u16, y as u16))
                .terrain = Terrain::Grassland;
        }
    }
    engine.game.spawn_unit(
        UnitClass::Settler,
        Location::new(14, 0),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine.game.spawn_unit(
        UnitClass::Settler,
        Location::new(6, 0),
        PlayerId::new(1),
        CityId::new(0),
    );
    engine.game.cities.push(City::new(
        "Tenochtitlan",
        Location::new(2, 1),
        PlayerId::new(2),
        CityId::new(1),
    ));
    engine.game.players[1].reveal_tiles_at(Location::new(7, 1), 8);

    engine.submit(Command::EndTurn);

    let zulu_cities: Vec<Location> = engine
        .game
        .cities
        .iter()
        .filter(|city| city.owner() == PlayerId::new(1))
        .map(|city| city.location)
        .collect();
    assert_eq!(zulu_cities.len(), 1, "the settler founds its capital");
    assert_eq!(zulu_cities[0], Location::new(6, 0));
    let shared = footprint_overlap_with(&engine, Location::new(6, 0), Location::new(2, 1));
    assert!(
        shared <= 3,
        "one rival keeps clear of another's grid (shared {shared})"
    );
    assert_eq!(engine.current_player(), Civilization::English);
}

#[test]
fn a_unit_can_move_only_once_per_turn() {
    let mut engine = test_engine();
    engine.game.map.tile_at_mut(Location::new(2, 1)).terrain = Terrain::Grassland;
    engine.submit(Command::Move {
        unit: UnitId::new(0),
        direction: Direction::E,
    });
    let events = engine.submit(Command::Move {
        unit: UnitId::new(0),
        direction: Direction::W,
    });
    assert_eq!(events[0].message(), "Unit 0 has no moves left");
    assert_eq!(engine.game.units[0].location, Location::new(2, 1));
}

#[test]
fn difficult_terrain_slows_a_fast_unit_to_one_tile_at_a_time() {
    let mut engine = Engine::new(5, 1, Player::new(Civilization::English), Vec::new());
    engine.game.spawn_unit(
        UnitClass::Cavalry,
        Location::new(0, 0),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine.game.map.tile_at_mut(Location::new(0, 0)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(1, 0)).terrain = Terrain::Forest;
    engine.game.map.tile_at_mut(Location::new(2, 0)).terrain = Terrain::Forest;
    // Grassland the unit stands on does not cost movement; entering the
    // second forest has an empty budget by then, so it stays put.
    engine.submit(Command::Move {
        unit: UnitId::new(0),
        direction: Direction::E,
    });
    assert_eq!(engine.game.units[0].location, Location::new(1, 0));
    // Cavalry has 3 moves; a forest costs 2, so one tile leaves 1 left.
    assert_eq!(engine.game.units[0].moves_remaining(), 1);
    // Entering the second forest with only 1 left still moves but drains
    // the budget to zero (move first, then deduct the full cost).
    engine.submit(Command::Move {
        unit: UnitId::new(0),
        direction: Direction::E,
    });
    assert_eq!(engine.game.units[0].location, Location::new(2, 0));
    assert_eq!(engine.game.units[0].moves_remaining(), 0);
    // With no movement left the unit cannot take another tile.
    let events = engine.submit(Command::Move {
        unit: UnitId::new(0),
        direction: Direction::E,
    });
    assert_eq!(events[0].message(), "Unit 0 has no moves left");
    assert_eq!(engine.game.units[0].location, Location::new(2, 0));
}

#[test]
fn a_slow_unit_can_enter_difficult_terrain_at_cost_of_its_movement() {
    let mut engine = Engine::new(5, 1, Player::new(Civilization::English), Vec::new());
    engine.game.spawn_unit(
        UnitClass::Settler,
        Location::new(0, 0),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine.game.map.tile_at_mut(Location::new(1, 0)).terrain = Terrain::Forest;
    // A settler has 1 move; entering the forest still happens because the
    // move is made first, then the 2-cost forest drains the budget.
    engine.submit(Command::Move {
        unit: UnitId::new(0),
        direction: Direction::E,
    });
    assert_eq!(engine.game.units[0].location, Location::new(1, 0));
    assert_eq!(engine.game.units[0].moves_remaining(), 0);
}

#[test]
fn moves_are_restored_at_the_beginning_of_the_owners_turn() {
    let mut engine = test_engine();
    engine.game.map.tile_at_mut(Location::new(2, 1)).terrain = Terrain::Grassland;
    engine.submit(Command::Move {
        unit: UnitId::new(0),
        direction: Direction::E,
    });
    assert_eq!(engine.game.units[0].moves_remaining(), 0);
    engine.submit(Command::EndTurn);
    assert_eq!(engine.game.units[0].moves_remaining(), 1);
}

#[test]
fn each_players_units_reset_when_their_turn_begins() {
    let mut engine = two_player_engine();
    let zulu_warrior = engine.game.spawn_unit(
        UnitClass::Legion,
        Location::new(0, 0),
        PlayerId::new(1),
        CityId::new(0),
    );
    engine.game.map.tile_at_mut(Location::new(2, 1)).terrain = Terrain::Grassland;
    engine.submit(Command::Move {
        unit: UnitId::new(0),
        direction: Direction::E,
    });
    assert_eq!(engine.game.units[0].moves_remaining(), 0);
    // The round plays the rival's turn and then the human's is begun afresh,
    // so every unit — the spent settler and the rival's standby legion — is
    // back to full moves when control returns.
    engine.submit(Command::EndTurn);
    assert_eq!(engine.current_player(), Civilization::English);
    assert_eq!(engine.game.units[0].moves_remaining(), 1);
    assert_eq!(
        engine
            .game
            .units
            .iter()
            .find(|unit| unit.id() == zulu_warrior)
            .unwrap()
            .moves_remaining(),
        1
    );
}

#[test]
fn a_land_unit_cannot_enter_water() {
    let mut engine = test_engine();
    let events = engine.submit(Command::Move {
        unit: UnitId::new(0),
        direction: Direction::E,
    });
    assert_eq!(events[0].message(), "Unit 0 cannot board: no ship there");
    assert_eq!(engine.game.units[0].location, Location::new(1, 1));
    assert_eq!(engine.game.units[0].moves_remaining(), 1);
}

#[test]
fn a_naval_unit_can_enter_water() {
    let mut engine = test_engine();
    engine.game.spawn_unit(
        UnitClass::Trireme,
        Location::new(1, 0),
        PlayerId::new(0),
        CityId::new(0),
    );
    let events = engine.submit(Command::Move {
        unit: UnitId::new(1),
        direction: Direction::S,
    });
    assert_eq!(events[0].message(), "Unit 1 moves S");
    assert_eq!(engine.game.units[1].location, Location::new(1, 1));
}

#[test]
fn a_naval_unit_cannot_enter_land() {
    let mut engine = test_engine();
    engine.game.spawn_unit(
        UnitClass::Trireme,
        Location::new(2, 0),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine.game.map.tile_at_mut(Location::new(2, 1)).terrain = Terrain::Grassland;
    let events = engine.submit(Command::Move {
        unit: UnitId::new(1),
        direction: Direction::S,
    });
    assert_eq!(events[0].message(), "Unit 1 cannot cross land/sea border");
    assert_eq!(engine.game.units[1].location, Location::new(2, 0));
    assert_eq!(engine.game.units[1].moves_remaining(), 3);
}

#[test]
fn end_turn_after_the_last_player_wraps_and_advances_the_turn() {
    let mut engine = two_player_engine();
    engine.submit(Command::EndTurn);
    let events = engine.submit(Command::EndTurn);
    assert_eq!(engine.current_player(), Civilization::English);
    assert_eq!(engine.turn(), 3);
    assert_eq!(events[0].message(), "Zulu begins turn 2");
    assert_eq!(events.last().unwrap().message(), "English begins turn 3");
}

#[test]
fn three_players_rotate_through_three_full_turns() {
    let mut engine = three_player_engine();
    // Each EndTurn resolves the whole round: both rivals act, then the human
    // starts the next turn.
    for round in 1..=3 {
        let events = engine.submit(Command::EndTurn);
        assert_eq!(events[0].message(), format!("Zulu begins turn {}", round));
        assert_eq!(
            events.last().unwrap().message(),
            format!("English begins turn {}", round + 1)
        );
        assert_eq!(engine.current_player(), Civilization::English);
    }
    assert_eq!(engine.turn(), 4);
}

#[test]
fn events_accumulate_only_within_a_single_submit() {
    let mut engine = test_engine();
    engine.submit(Command::EndTurn);
    let events = engine.submit(Command::Fortify {
        unit: UnitId::new(0),
    });
    assert_eq!(events.len(), 1);
}

#[test]
fn a_settler_founds_a_city() {
    let mut engine = Engine::new(5, 5, Player::new(Civilization::English), Vec::new());
    engine.game.map.tile_at_mut(Location::new(2, 2)).terrain = Terrain::Grassland;
    let settler = engine.game.spawn_unit(
        UnitClass::Settler,
        Location::new(2, 2),
        PlayerId::new(0),
        CityId::new(0),
    );
    let events = engine.submit(Command::FoundCity {
        unit: settler,
        name: "London".to_string(),
    });
    assert_eq!(
        events[0].message(),
        "English begin researching Construction"
    );
    assert_eq!(events[1].message(), "Unit 0 founds London");
    assert!(engine.game.units.is_empty());
    assert_eq!(engine.game.cities.len(), 1);
    assert_eq!(engine.game.cities[0].name, "London");
    assert_eq!(engine.game.cities[0].location, Location::new(2, 2));
    assert_eq!(engine.game.cities[0].owner(), PlayerId::new(0));
    assert_eq!(
        engine.game.advancement_in_progress(PlayerId::new(0)),
        Some(Advancement::Construction)
    );
}

#[test]
fn founding_a_city_reveals_tiles_around_it_for_its_owner() {
    let mut engine = two_player_engine();
    engine.game.map.tile_at_mut(Location::new(1, 1)).terrain = Terrain::Grassland;
    let settler = engine.game.spawn_unit(
        UnitClass::Settler,
        Location::new(1, 1),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine.submit(Command::FoundCity {
        unit: settler,
        name: "London".to_string(),
    });
    assert!(engine.game.players[0].explored_at(1, 1));
    assert!(engine.game.players[0].explored_at(0, 0));
    assert!(!engine.game.players[1].explored_at(1, 1));
}

#[test]
fn founding_a_city_reveals_its_footprint_but_not_the_corners() {
    let mut engine = Engine::new(5, 5, Player::new(Civilization::English), Vec::new());
    engine.game.map.tile_at_mut(Location::new(2, 2)).terrain = Terrain::Grassland;
    let settler = engine.game.spawn_unit(
        UnitClass::Settler,
        Location::new(2, 2),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine.submit(Command::FoundCity {
        unit: settler,
        name: "London".to_string(),
    });
    assert!(engine.game.players[0].explored_at(0, 2));
    assert!(engine.game.players[0].explored_at(2, 4));
    assert!(!engine.game.players[0].explored_at(0, 0));
    assert!(!engine.game.players[0].explored_at(4, 4));
}

#[test]
fn non_settlers_cannot_found_cities() {
    let mut engine = test_engine();
    engine.game.spawn_unit(
        UnitClass::Legion,
        Location::new(1, 1),
        PlayerId::new(0),
        CityId::new(0),
    );
    let events = engine.submit(Command::FoundCity {
        unit: UnitId::new(1),
        name: "London".to_string(),
    });
    assert_eq!(events[0].message(), "Unit 1 cannot found a city");
    assert!(engine.game.cities.is_empty());
    assert!(!engine.game.units.is_empty());
}

#[test]
fn cities_cannot_be_founded_on_water() {
    let mut engine = test_engine();
    let events = engine.submit(Command::FoundCity {
        unit: UnitId::new(0),
        name: "London".to_string(),
    });
    assert_eq!(
        events[0].message(),
        "Unit 0 must be on land to found a city"
    );
    assert!(engine.game.cities.is_empty());
    assert_eq!(engine.game.units.len(), 1);
}

#[test]
fn a_city_cannot_be_founded_where_a_city_already_exists() {
    let mut engine = Engine::new(5, 5, Player::new(Civilization::English), Vec::new());
    engine.game.map.tile_at_mut(Location::new(2, 2)).terrain = Terrain::Grassland;
    engine
        .game
        .add_city(PlayerId::new(0), "London", Location::new(2, 2));
    let settler = engine.game.spawn_unit(
        UnitClass::Settler,
        Location::new(2, 2),
        PlayerId::new(0),
        CityId::new(0),
    );
    let events = engine.submit(Command::FoundCity {
        unit: settler,
        name: "York".to_string(),
    });
    assert_eq!(events[0].message(), "A city already occupies that tile");
    assert_eq!(engine.game.cities.len(), 1);
    assert_eq!(engine.game.units.len(), 1);
}

#[test]
fn founding_with_an_unknown_unit_is_rejected() {
    let mut engine = test_engine();
    let events = engine.submit(Command::FoundCity {
        unit: UnitId::new(99),
        name: "Atlantis".to_string(),
    });
    assert_eq!(events[0].message(), "No such unit");
}

fn blank_war_map() -> Engine {
    let mut engine = Engine::with_seed(
        5,
        5,
        Player::new(Civilization::English),
        vec![Player::new(Civilization::Zulu)],
        11,
    );
    engine.game.declare_war(PlayerId::new(0), PlayerId::new(1));
    engine
}

#[test]
fn attacker_defeats_the_target_and_takes_its_tile() {
    let mut engine = blank_war_map();
    engine.game.map.tile_at_mut(Location::new(2, 2)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(3, 2)).terrain = Terrain::Grassland;
    let legion = engine.game.spawn_unit(
        UnitClass::Legion,
        Location::new(2, 2),
        PlayerId::new(0),
        CityId::new(0),
    );
    let militia = engine.game.spawn_unit(
        UnitClass::Militia,
        Location::new(3, 2),
        PlayerId::new(1),
        CityId::new(1),
    );
    let events = engine.submit(Command::Move {
        unit: legion,
        direction: Direction::E,
    });
    assert_eq!(
        events[0].message(),
        format!("Unit {} attacks Unit {}", legion.index(), militia.index())
    );
    assert_eq!(
        events[1].message(),
        format!("Unit {} defeats Unit {}", legion.index(), militia.index())
    );
    assert!(!engine.game.units.iter().any(|unit| unit.id() == militia));
    let legion_unit = engine
        .game
        .units
        .iter()
        .find(|unit| unit.id() == legion)
        .unwrap();
    assert_eq!(legion_unit.location, Location::new(3, 2));
    assert_eq!(legion_unit.moves_remaining(), 0);
    assert!(legion_unit.is_veteran());
}

#[test]
fn an_at_war_unit_moving_onto_an_undefended_enemy_city_captures_it() {
    let mut engine = blank_war_map();
    engine.game.map.tile_at_mut(Location::new(2, 2)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(3, 2)).terrain = Terrain::Grassland;
    engine
        .game
        .add_city(PlayerId::new(1), "Umgungundlovu", Location::new(3, 2));
    // A population-one city is razed rather than captured, so grow it to
    // size two to exercise the capture itself.
    engine.game.cities[0].grow();
    let legion = engine.game.spawn_unit(
        UnitClass::Legion,
        Location::new(2, 2),
        PlayerId::new(0),
        CityId::new(2),
    );
    let events = engine.submit(Command::Move {
        unit: legion,
        direction: Direction::E,
    });
    assert!(
        events
            .iter()
            .any(|e| e.message() == "English capture Umgungundlovu (formerly Zulu's)")
    );
    let city = engine
        .game
        .cities
        .iter()
        .find(|c| c.name == "Umgungundlovu")
        .unwrap();
    assert_eq!(city.owner(), PlayerId::new(0));
    let unit = engine.game.units.iter().find(|u| u.id() == legion).unwrap();
    assert_eq!(unit.location, Location::new(3, 2));
    assert_eq!(unit.moves_remaining(), 0);
}

#[test]
fn capturing_a_city_reveals_the_tiles_around_it_for_the_conqueror() {
    let mut engine = blank_war_map();
    engine.game.map.tile_at_mut(Location::new(2, 2)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(3, 2)).terrain = Terrain::Grassland;
    engine
        .game
        .add_city(PlayerId::new(1), "Umgungundlovu", Location::new(3, 2));
    // Grow it so the capture (not the pop-one raze) is what is tested.
    engine.game.cities[0].grow();
    let legion = engine.game.spawn_unit(
        UnitClass::Legion,
        Location::new(2, 2),
        PlayerId::new(0),
        CityId::new(2),
    );
    // The legion's starting reveal (radius 1 around (2, 2)) stops at
    // column 3; the ring east of the conquered city is still dark.
    assert!(!engine.explored(4, 1));
    assert!(!engine.explored(4, 2));
    engine.submit(Command::Move {
        unit: legion,
        direction: Direction::E,
    });
    // Advancing onto the captured city reveals its surroundings as if the
    // unit had simply occupied the tiles itself.
    assert!(engine.explored(4, 1));
    assert!(engine.explored(4, 2));
    assert!(engine.explored(4, 3));
    assert!(!engine.explored(4, 4));
}

#[test]
fn a_winning_attacker_reveals_the_tiles_it_advances_onto() {
    let mut engine = blank_war_map();
    engine.game.map.tile_at_mut(Location::new(2, 2)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(3, 2)).terrain = Terrain::Grassland;
    let legion = engine.game.spawn_unit(
        UnitClass::Legion,
        Location::new(2, 2),
        PlayerId::new(0),
        CityId::new(0),
    );
    let militia = engine.game.spawn_unit(
        UnitClass::Militia,
        Location::new(3, 2),
        PlayerId::new(1),
        CityId::new(1),
    );
    assert!(!engine.explored(4, 2));
    engine.submit(Command::Move {
        unit: legion,
        direction: Direction::E,
    });
    assert!(!engine.game.units.iter().any(|unit| unit.id() == militia));
    // Surviving the battle and moving onto the target tile reveals the
    // ring of tiles around the unit's new position.
    assert!(engine.explored(4, 1));
    assert!(engine.explored(4, 2));
    assert!(engine.explored(4, 3));
    assert!(!engine.explored(4, 4));
}

#[test]
fn capturing_a_city_disbands_units_homed_to_it() {
    let mut engine = blank_war_map();
    engine.game.map.tile_at_mut(Location::new(2, 2)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(3, 2)).terrain = Terrain::Grassland;
    engine
        .game
        .add_city(PlayerId::new(1), "Umgungundlovu", Location::new(3, 2));
    engine.game.cities[0].grow();
    // A second Zulu city (id 1) survives, so the civilization is not
    // eliminated and only the units homed to the captured city disband.
    engine
        .game
        .add_city(PlayerId::new(1), "Ulundi", Location::new(4, 2));
    let legion = engine.game.spawn_unit(
        UnitClass::Legion,
        Location::new(2, 2),
        PlayerId::new(0),
        CityId::new(2),
    );
    // A Zulu unit homed to the captured city (id 0) sits elsewhere on the map.
    let lost_phalanx = engine.game.spawn_unit(
        UnitClass::Phalanx,
        Location::new(0, 0),
        PlayerId::new(1),
        CityId::new(0),
    );
    // An unrelated Zulu unit homed to a different city survives.
    let other_phalanx = engine.game.spawn_unit(
        UnitClass::Phalanx,
        Location::new(0, 1),
        PlayerId::new(1),
        CityId::new(1),
    );
    let events = engine.submit(Command::Move {
        unit: legion,
        direction: Direction::E,
    });
    assert!(
        events
            .iter()
            .any(|e| e.message() == "1 units disband with the loss of Umgungundlovu")
    );
    assert!(!engine.game.units.iter().any(|u| u.id() == lost_phalanx));
    assert!(engine.game.units.iter().any(|u| u.id() == other_phalanx));
}

#[test]
fn capturing_a_civilizations_last_city_removes_it_from_play() {
    let mut engine = blank_war_map();
    engine.game.map.tile_at_mut(Location::new(2, 2)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(3, 2)).terrain = Terrain::Grassland;
    engine
        .game
        .add_city(PlayerId::new(1), "Umgungundlovu", Location::new(3, 2));
    engine.game.cities[0].grow();
    let legion = engine.game.spawn_unit(
        UnitClass::Legion,
        Location::new(2, 2),
        PlayerId::new(0),
        CityId::new(9),
    );
    // A surviving Zulu unit lies elsewhere on the map: losing the last
    // city still ends the civilization.
    let stray_phalanx = engine.game.spawn_unit(
        UnitClass::Phalanx,
        Location::new(0, 0),
        PlayerId::new(1),
        CityId::new(1),
    );
    let events = engine.submit(Command::Move {
        unit: legion,
        direction: Direction::E,
    });
    assert!(
        events
            .iter()
            .any(|e| e.message() == "Zulu has been eliminated")
    );
    assert!(
        events
            .iter()
            .any(|e| e.message() == "1 units disband with the loss of Zulu")
    );
    assert!(engine.game.players[1].eliminated());
    assert!(!engine.game.units.iter().any(|u| u.id() == stray_phalanx));
}

#[test]
fn losing_a_city_but_keeping_another_leaves_the_civilization_in_play() {
    let mut engine = blank_war_map();
    engine.game.map.tile_at_mut(Location::new(2, 2)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(3, 2)).terrain = Terrain::Grassland;
    engine
        .game
        .add_city(PlayerId::new(1), "Umgungundlovu", Location::new(3, 2));
    engine.game.cities[0].grow();
    engine
        .game
        .add_city(PlayerId::new(1), "Ulundi", Location::new(4, 2));
    let legion = engine.game.spawn_unit(
        UnitClass::Legion,
        Location::new(2, 2),
        PlayerId::new(0),
        CityId::new(9),
    );
    engine.submit(Command::Move {
        unit: legion,
        direction: Direction::E,
    });
    assert!(!engine.game.players[1].eliminated());
    assert_eq!(
        engine
            .game
            .cities
            .iter()
            .filter(|c| c.owner() == PlayerId::new(1))
            .count(),
        1
    );
}

#[test]
fn annihilating_a_cityless_civilization_removes_it_from_play() {
    let mut engine = blank_war_map();
    engine.game.map.tile_at_mut(Location::new(2, 2)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(3, 2)).terrain = Terrain::Grassland;
    let legion = engine.game.spawn_unit(
        UnitClass::Legion,
        Location::new(2, 2),
        PlayerId::new(0),
        CityId::new(0),
    );
    // Zulu has no cities and sits on the map with a single militia.
    let militia = engine.game.spawn_unit(
        UnitClass::Militia,
        Location::new(3, 2),
        PlayerId::new(1),
        CityId::new(1),
    );
    let events = engine.submit(Command::Move {
        unit: legion,
        direction: Direction::E,
    });
    // Blank_war_map's seed sends the legion through the militia; killing
    // Zulu's only unit while they hold no cities ends the civilization.
    assert!(!engine.game.units.iter().any(|u| u.id() == militia));
    assert!(engine.game.players[1].eliminated());
    assert!(
        events
            .iter()
            .any(|e| e.message() == "Zulu has been eliminated")
    );
}

#[test]
fn a_settler_with_no_cities_survives_the_loss_of_other_units() {
    let mut engine = blank_war_map();
    engine.game.map.tile_at_mut(Location::new(2, 2)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(3, 2)).terrain = Terrain::Grassland;
    let legion = engine.game.spawn_unit(
        UnitClass::Legion,
        Location::new(2, 2),
        PlayerId::new(0),
        CityId::new(0),
    );
    // Zulu has no cities but a militia *and* a settler still in the field.
    engine.game.spawn_unit(
        UnitClass::Militia,
        Location::new(3, 2),
        PlayerId::new(1),
        CityId::new(1),
    );
    let settler = engine.game.spawn_unit(
        UnitClass::Settler,
        Location::new(0, 0),
        PlayerId::new(1),
        CityId::new(1),
    );
    engine.submit(Command::Move {
        unit: legion,
        direction: Direction::E,
    });
    // Whatever the combat result, the cityless civilization is
    // not yet out: it still holds the settler and may found a city.
    assert!(!engine.game.players[1].eliminated());
    assert!(engine.game.units.iter().any(|u| u.id() == settler));
}

#[test]
fn an_eliminated_civilization_is_skipped_when_the_turn_advances() {
    let mut engine = Engine::new(
        5,
        5,
        Player::new(Civilization::English),
        vec![
            Player::new(Civilization::Zulu),
            Player::new(Civilization::Roman),
        ],
    );
    engine.game.declare_war(PlayerId::new(0), PlayerId::new(1));
    engine.game.map.tile_at_mut(Location::new(2, 2)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(3, 2)).terrain = Terrain::Grassland;
    engine
        .game
        .add_city(PlayerId::new(1), "Umgungundlovu", Location::new(3, 2));
    engine.game.cities[0].grow();
    let legion = engine.game.spawn_unit(
        UnitClass::Legion,
        Location::new(2, 2),
        PlayerId::new(0),
        CityId::new(9),
    );
    // Capture Zulu's only city; play moves on to the surviving rival, then
    // the round returns to English with the turn advanced. Zulu never takes
    // another turn.
    engine.submit(Command::Move {
        unit: legion,
        direction: Direction::E,
    });
    assert!(engine.game.players[1].eliminated());
    let events = engine.submit(Command::EndTurn);
    assert_eq!(engine.current_player(), Civilization::English);
    assert_eq!(engine.turn(), 2);
    assert!(events.iter().any(|e| e.message() == "Roman begins turn 1"));
    assert!(
        !events
            .iter()
            .any(|e| e.message().starts_with("Zulu begins"))
    );
    assert!(
        events
            .iter()
            .any(|e| e.message() == "English begins turn 2")
    );
}

#[test]
fn winning_a_fight_on_a_city_tile_captures_the_city() {
    let mut engine = blank_war_map();
    engine.game.map.tile_at_mut(Location::new(2, 2)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(3, 2)).terrain = Terrain::Grassland;
    engine
        .game
        .add_city(PlayerId::new(1), "Umgungundlovu", Location::new(3, 2));
    engine.game.cities[0].grow();
    let legion = engine.game.spawn_unit(
        UnitClass::Legion,
        Location::new(2, 2),
        PlayerId::new(0),
        CityId::new(9),
    );
    // A militia's meagre defence keeps the fight short; the win then advances
    // the legion onto the city tile, where the city is now his.
    let militia = engine.game.spawn_unit(
        UnitClass::Militia,
        Location::new(3, 2),
        PlayerId::new(1),
        CityId::new(1),
    );
    let events = engine.submit(Command::Move {
        unit: legion,
        direction: Direction::E,
    });
    assert!(events.iter().any(
        |e| e.message() == format!("Unit {} defeats Unit {}", legion.index(), militia.index())
    ));
    assert!(!engine.game.units.iter().any(|unit| unit.id() == militia));
    assert!(
        events
            .iter()
            .any(|e| e.message() == "English capture Umgungundlovu (formerly Zulu's)"),
        "winning the fight takes the city on the spot: {:?}",
        events.iter().map(|e| e.message()).collect::<Vec<_>>()
    );
    let city = engine
        .game
        .cities
        .iter()
        .find(|c| c.name == "Umgungundlovu")
        .unwrap();
    assert_eq!(city.owner(), PlayerId::new(0));
    let legion_unit = engine
        .game
        .units
        .iter()
        .find(|unit| unit.id() == legion)
        .unwrap();
    assert_eq!(legion_unit.location, Location::new(3, 2));
}

#[test]
fn conquering_a_population_one_city_destroys_it() {
    let mut engine = blank_war_map();
    engine.game.map.tile_at_mut(Location::new(2, 2)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(3, 2)).terrain = Terrain::Grassland;
    engine
        .game
        .add_city(PlayerId::new(1), "Umgungundlovu", Location::new(3, 2));
    let legion = engine.game.spawn_unit(
        UnitClass::Legion,
        Location::new(2, 2),
        PlayerId::new(0),
        CityId::new(9),
    );
    let events = engine.submit(Command::Move {
        unit: legion,
        direction: Direction::E,
    });
    assert!(
        events
            .iter()
            .any(|e| e.message() == "English destroy Umgungundlovu (formerly Zulu's)"),
        "expected the razing event, got {:?}",
        events.iter().map(|e| e.message()).collect::<Vec<_>>()
    );
    assert!(
        !engine.game.cities.iter().any(|c| c.name == "Umgungundlovu"),
        "the destroyed city is removed from the game"
    );
    // Zulu holds no other city and no units: the razing ends the civilization.
    assert!(engine.game.players[1].eliminated());
    let legion_unit = engine
        .game
        .units
        .iter()
        .find(|unit| unit.id() == legion)
        .unwrap();
    assert_eq!(legion_unit.location, Location::new(3, 2));
}

#[test]
fn razing_a_population_one_city_leaves_a_surviving_civilization_in_play() {
    let mut engine = blank_war_map();
    engine.game.map.tile_at_mut(Location::new(2, 2)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(3, 2)).terrain = Terrain::Grassland;
    engine
        .game
        .add_city(PlayerId::new(1), "Umgungundlovu", Location::new(3, 2));
    engine
        .game
        .add_city(PlayerId::new(1), "Ulundi", Location::new(4, 2));
    let legion = engine.game.spawn_unit(
        UnitClass::Legion,
        Location::new(2, 2),
        PlayerId::new(0),
        CityId::new(9),
    );
    let events = engine.submit(Command::Move {
        unit: legion,
        direction: Direction::E,
    });
    assert!(
        events
            .iter()
            .any(|e| e.message() == "English destroy Umgungundlovu (formerly Zulu's)")
    );
    assert!(!engine.game.cities.iter().any(|c| c.name == "Umgungundlovu"));
    // Ulundi survives, so Zulu is not eliminated by the loss of one town.
    assert!(!engine.game.players[1].eliminated());
    assert_eq!(
        engine
            .game
            .cities
            .iter()
            .filter(|c| c.owner() == PlayerId::new(1))
            .count(),
        1
    );
}

#[test]
fn a_unit_at_peace_with_the_city_owner_cannot_move_into_the_city() {
    // two_player_engine is not at war: the two civilizations are unmet/peaceful.
    let mut engine = two_player_engine();
    engine.game.map.tile_at_mut(Location::new(1, 1)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(2, 1)).terrain = Terrain::Grassland;
    engine
        .game
        .add_city(PlayerId::new(1), "Umgungundlovu", Location::new(2, 1));
    let legion = engine.game.spawn_unit(
        UnitClass::Legion,
        Location::new(1, 1),
        PlayerId::new(0),
        CityId::new(0),
    );
    let events = engine.submit(Command::Move {
        unit: legion,
        direction: Direction::E,
    });
    assert!(events.iter().any(|e| e.message()
        == format!(
            "Unit {} cannot move onto a tile occupied by a civilization at peace",
            legion.index()
        )));
    // Neither ownership nor the unit's position change.
    let city = engine
        .game
        .cities
        .iter()
        .find(|c| c.name == "Umgungundlovu")
        .unwrap();
    assert_eq!(city.owner(), PlayerId::new(1));
    let unit = engine.game.units.iter().find(|u| u.id() == legion).unwrap();
    assert_eq!(unit.location, Location::new(1, 1));
}

#[test]
fn the_strongest_defender_on_the_target_tile_absorbs_the_attack() {
    let mut engine = Engine::with_seed(
        5,
        5,
        Player::new(Civilization::English),
        vec![Player::new(Civilization::Zulu)],
        1,
    );
    engine.game.declare_war(PlayerId::new(0), PlayerId::new(1));
    engine.game.map.tile_at_mut(Location::new(2, 2)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(3, 2)).terrain = Terrain::Grassland;
    let legion = engine.game.spawn_unit(
        UnitClass::Legion,
        Location::new(2, 2),
        PlayerId::new(0),
        CityId::new(0),
    );
    let settler = engine.game.spawn_unit(
        UnitClass::Settler,
        Location::new(3, 2),
        PlayerId::new(1),
        CityId::new(1),
    );
    let phalanx = engine.game.spawn_unit(
        UnitClass::Phalanx,
        Location::new(3, 2),
        PlayerId::new(1),
        CityId::new(1),
    );
    let events = engine.submit(Command::Move {
        unit: legion,
        direction: Direction::E,
    });
    assert_eq!(
        events[0].message(),
        format!("Unit {} attacks Unit {}", legion.index(), phalanx.index())
    );
    assert_eq!(
        events[1].message(),
        format!("Unit {} defeats Unit {}", legion.index(), phalanx.index())
    );
    assert!(!engine.game.units.iter().any(|unit| unit.id() == phalanx));
    assert!(engine.game.units.iter().any(|unit| unit.id() == settler));
    let legion_unit = engine
        .game
        .units
        .iter()
        .find(|unit| unit.id() == legion)
        .unwrap();
    assert_eq!(legion_unit.location, Location::new(2, 2));
    assert!(legion_unit.is_veteran());
}

#[test]
fn equal_defence_is_broken_in_favour_of_the_higher_unit_id() {
    let mut engine = blank_war_map();
    engine.game.map.tile_at_mut(Location::new(2, 2)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(3, 2)).terrain = Terrain::Grassland;
    let legion = engine.game.spawn_unit(
        UnitClass::Legion,
        Location::new(2, 2),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine.game.spawn_unit(
        UnitClass::Militia,
        Location::new(3, 2),
        PlayerId::new(1),
        CityId::new(1),
    );
    let stronger_id = engine.game.spawn_unit(
        UnitClass::Militia,
        Location::new(3, 2),
        PlayerId::new(1),
        CityId::new(1),
    );
    let events = engine.submit(Command::Move {
        unit: legion,
        direction: Direction::E,
    });
    assert_eq!(
        events[0].message(),
        format!(
            "Unit {} attacks Unit {}",
            legion.index(),
            stronger_id.index()
        )
    );
}

#[test]
fn a_repelled_attacker_is_removed_but_the_target_survives() {
    let mut engine = blank_war_map();
    engine.game.map.tile_at_mut(Location::new(2, 2)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(2, 3)).terrain = Terrain::Grassland;
    let militia = engine.game.spawn_unit(
        UnitClass::Militia,
        Location::new(2, 2),
        PlayerId::new(0),
        CityId::new(0),
    );
    let knight = engine.game.spawn_unit(
        UnitClass::Knight,
        Location::new(2, 3),
        PlayerId::new(1),
        CityId::new(1),
    );
    let events = engine.submit(Command::Move {
        unit: militia,
        direction: Direction::S,
    });
    assert_eq!(
        events[1].message(),
        format!("Unit {} repels Unit {}", knight.index(), militia.index())
    );
    assert!(!engine.game.units.iter().any(|unit| unit.id() == militia));
    let knight_unit = engine
        .game
        .units
        .iter()
        .find(|unit| unit.id() == knight)
        .unwrap();
    assert!(!knight_unit.is_veteran());
}

#[test]
fn moving_onto_a_friendly_unit_is_a_plain_move() {
    let mut engine = Engine::new(
        5,
        5,
        Player::new(Civilization::English),
        vec![Player::new(Civilization::Zulu)],
    );
    engine.game.map.tile_at_mut(Location::new(2, 2)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(3, 2)).terrain = Terrain::Grassland;
    let legion = engine.game.spawn_unit(
        UnitClass::Legion,
        Location::new(2, 2),
        PlayerId::new(0),
        CityId::new(0),
    );
    let phalanx = engine.game.spawn_unit(
        UnitClass::Phalanx,
        Location::new(3, 2),
        PlayerId::new(0),
        CityId::new(0),
    );
    let events = engine.submit(Command::Move {
        unit: legion,
        direction: Direction::E,
    });
    assert_eq!(
        events[0].message(),
        format!("Unit {} moves E", legion.index())
    );
    assert!(engine.game.units.iter().any(|unit| unit.id() == phalanx));
    let legion_unit = engine
        .game
        .units
        .iter()
        .find(|unit| unit.id() == legion)
        .unwrap();
    assert_eq!(legion_unit.location, Location::new(3, 2));
}

#[test]
fn attacker_power_is_base_attack_scaled_by_ten() {
    let mut engine = Engine::new(5, 5, Player::new(Civilization::English), Vec::new());
    engine.game.spawn_unit(
        UnitClass::Militia,
        Location::new(2, 2),
        PlayerId::new(0),
        CityId::new(0),
    );
    assert_eq!(engine.attacker_power(&engine.game.units[0]), 10);
    engine.game.spawn_unit(
        UnitClass::Legion,
        Location::new(2, 3),
        PlayerId::new(0),
        CityId::new(0),
    );
    assert_eq!(engine.attacker_power(&engine.game.units[1]), 30);
}

#[test]
fn veteran_attacks_at_half_again_power() {
    let mut engine = Engine::new(5, 5, Player::new(Civilization::English), Vec::new());
    engine.game.spawn_unit(
        UnitClass::Militia,
        Location::new(2, 2),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine.game.units[0].promote();
    assert_eq!(engine.attacker_power(&engine.game.units[0]), 15);
}

#[test]
fn defender_power_applies_terrain_city_and_veteran_bonuses() {
    let mut engine = Engine::new(
        5,
        5,
        Player::new(Civilization::English),
        vec![Player::new(Civilization::Zulu)],
    );
    engine.game.map.tile_at_mut(Location::new(2, 2)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(2, 3)).terrain = Terrain::Mountain;
    engine.game.spawn_unit(
        UnitClass::Militia,
        Location::new(2, 2),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine.game.spawn_unit(
        UnitClass::Militia,
        Location::new(2, 3),
        PlayerId::new(1),
        CityId::new(1),
    );
    assert_eq!(engine.defender_power(&engine.game.units[0]), 10);
    assert_eq!(engine.defender_power(&engine.game.units[1]), 20);
    engine
        .game
        .add_city(PlayerId::new(1), "Umgungundlovu", Location::new(2, 3));
    assert_eq!(engine.defender_power(&engine.game.units[1]), 30);
    engine.game.units[1].promote();
    assert_eq!(engine.defender_power(&engine.game.units[1]), 45);
}

#[test]
fn movement_onto_a_peaceful_tile_is_blocked() {
    let mut engine = Engine::new(
        5,
        5,
        Player::new(Civilization::English),
        vec![Player::new(Civilization::Zulu)],
    );
    engine.game.map.tile_at_mut(Location::new(2, 2)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(3, 2)).terrain = Terrain::Grassland;
    let legion = engine.game.spawn_unit(
        UnitClass::Legion,
        Location::new(2, 2),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine.game.spawn_unit(
        UnitClass::Militia,
        Location::new(3, 2),
        PlayerId::new(1),
        CityId::new(1),
    );
    let events = engine.submit(Command::Move {
        unit: legion,
        direction: Direction::E,
    });
    assert_eq!(
        events[0].message(),
        format!(
            "Unit {} cannot move onto a tile occupied by a civilization at peace",
            legion.index()
        )
    );
    let legion_unit = engine
        .game
        .units
        .iter()
        .find(|unit| unit.id() == legion)
        .unwrap();
    assert_eq!(legion_unit.location, Location::new(2, 2));
}

#[test]
fn declaring_war_makes_enemy_tiles_attackable() {
    let mut engine = Engine::new(
        5,
        5,
        Player::new(Civilization::English),
        vec![Player::new(Civilization::Zulu)],
    );
    engine.game.map.tile_at_mut(Location::new(2, 2)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(3, 2)).terrain = Terrain::Grassland;
    let legion = engine.game.spawn_unit(
        UnitClass::Legion,
        Location::new(2, 2),
        PlayerId::new(0),
        CityId::new(0),
    );
    let militia = engine.game.spawn_unit(
        UnitClass::Militia,
        Location::new(3, 2),
        PlayerId::new(1),
        CityId::new(1),
    );
    let war_events = engine.submit(Command::DeclareWar {
        opponent: PlayerId::new(1),
    });
    assert_eq!(war_events[0].message(), "English declares war on Zulu");
    let events = engine.submit(Command::Move {
        unit: legion,
        direction: Direction::E,
    });
    assert_eq!(
        events[1].message(),
        format!("Unit {} defeats Unit {}", legion.index(), militia.index())
    );
}

#[test]
fn declaring_war_on_yourself_or_a_phantom_player_is_rejected() {
    let mut engine = Engine::new(
        5,
        5,
        Player::new(Civilization::English),
        vec![Player::new(Civilization::Zulu)],
    );
    let events = engine.submit(Command::DeclareWar {
        opponent: PlayerId::new(0),
    });
    assert_eq!(events[0].message(), "Cannot declare war on yourself");
    let events = engine.submit(Command::DeclareWar {
        opponent: PlayerId::new(7),
    });
    assert_eq!(events[0].message(), "No such player");
}

#[test]
fn declaring_war_twice_is_redundant() {
    let mut engine = Engine::new(
        5,
        5,
        Player::new(Civilization::English),
        vec![Player::new(Civilization::Zulu)],
    );
    engine.submit(Command::DeclareWar {
        opponent: PlayerId::new(1),
    });
    let events = engine.submit(Command::DeclareWar {
        opponent: PlayerId::new(1),
    });
    assert_eq!(events[0].message(), "Already at war");
}

#[test]
fn combat_only_engages_players_we_are_at_war_with() {
    let mut engine = Engine::with_seed(
        5,
        5,
        Player::new(Civilization::English),
        vec![
            Player::new(Civilization::Zulu),
            Player::new(Civilization::Roman),
        ],
        1,
    );
    engine.game.declare_war(PlayerId::new(0), PlayerId::new(2));
    engine.game.make_peace(PlayerId::new(0), PlayerId::new(1));
    engine.game.map.tile_at_mut(Location::new(2, 2)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(3, 2)).terrain = Terrain::Grassland;
    let legion = engine.game.spawn_unit(
        UnitClass::Legion,
        Location::new(2, 2),
        PlayerId::new(0),
        CityId::new(0),
    );
    let ally = engine.game.spawn_unit(
        UnitClass::Militia,
        Location::new(3, 2),
        PlayerId::new(1),
        CityId::new(1),
    );
    let enemy = engine.game.spawn_unit(
        UnitClass::Knight,
        Location::new(3, 2),
        PlayerId::new(2),
        CityId::new(2),
    );
    let events = engine.submit(Command::Move {
        unit: legion,
        direction: Direction::E,
    });
    assert_eq!(
        events[0].message(),
        format!("Unit {} attacks Unit {}", legion.index(), enemy.index())
    );
    assert_eq!(
        events[1].message(),
        format!("Unit {} defeats Unit {}", legion.index(), enemy.index())
    );
    assert!(!engine.game.units.iter().any(|unit| unit.id() == enemy));
    assert!(engine.game.units.iter().any(|unit| unit.id() == ally));
    let legion_unit = engine
        .game
        .units
        .iter()
        .find(|unit| unit.id() == legion)
        .unwrap();
    assert_eq!(legion_unit.location, Location::new(3, 2));
}

#[test]
fn making_peace_registers_and_blocks_movement_again() {
    let mut engine = Engine::new(
        5,
        5,
        Player::new(Civilization::English),
        vec![Player::new(Civilization::Zulu)],
    );
    engine.game.map.tile_at_mut(Location::new(2, 2)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(3, 2)).terrain = Terrain::Grassland;
    let legion = engine.game.spawn_unit(
        UnitClass::Legion,
        Location::new(2, 2),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine.game.spawn_unit(
        UnitClass::Militia,
        Location::new(3, 2),
        PlayerId::new(1),
        CityId::new(1),
    );
    engine.submit(Command::DeclareWar {
        opponent: PlayerId::new(1),
    });
    engine.submit(Command::MakePeace {
        opponent: PlayerId::new(1),
    });
    let events = engine.submit(Command::Move {
        unit: legion,
        direction: Direction::E,
    });
    assert_eq!(
        events[0].message(),
        format!(
            "Unit {} cannot move onto a tile occupied by a civilization at peace",
            legion.index()
        )
    );
}

#[test]
fn making_peace_with_yourself_a_phantom_or_a_friend_is_rejected() {
    let mut engine = Engine::new(
        5,
        5,
        Player::new(Civilization::English),
        vec![Player::new(Civilization::Zulu)],
    );
    let events = engine.submit(Command::MakePeace {
        opponent: PlayerId::new(0),
    });
    assert_eq!(events[0].message(), "Cannot make peace with yourself");
    let events = engine.submit(Command::MakePeace {
        opponent: PlayerId::new(7),
    });
    assert_eq!(events[0].message(), "No such player");
    let events = engine.submit(Command::MakePeace {
        opponent: PlayerId::new(1),
    });
    assert_eq!(events[0].message(), "English makes peace with Zulu");
    let events = engine.submit(Command::MakePeace {
        opponent: PlayerId::new(1),
    });
    assert_eq!(events[0].message(), "Already at peace");
}

#[test]
fn moving_adjacent_to_a_foreign_unit_establishes_first_contact() {
    let mut engine = Engine::new(
        5,
        5,
        Player::new(Civilization::English),
        vec![Player::new(Civilization::Zulu)],
    );
    engine.game.map.tile_at_mut(Location::new(1, 1)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(2, 1)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(3, 1)).terrain = Terrain::Grassland;
    let legion = engine.game.spawn_unit(
        UnitClass::Legion,
        Location::new(1, 1),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine.game.spawn_unit(
        UnitClass::Militia,
        Location::new(3, 1),
        PlayerId::new(1),
        CityId::new(1),
    );
    assert!(!engine.game.have_met(PlayerId::new(0), PlayerId::new(1)));
    let events = engine.submit(Command::Move {
        unit: legion,
        direction: Direction::E,
    });
    assert_eq!(
        events[0].message(),
        "English and Zulu meet for the first time"
    );
    assert!(engine.game.at_peace(PlayerId::new(0), PlayerId::new(1)));
    engine.submit(Command::EndTurn);
    engine.submit(Command::EndTurn);
    let events = engine.submit(Command::Move {
        unit: legion,
        direction: Direction::E,
    });
    assert_eq!(
        events[0].message(),
        format!(
            "Unit {} cannot move onto a tile occupied by a civilization at peace",
            legion.index()
        )
    );
}

#[test]
fn first_contact_with_a_foreign_city_makes_peace() {
    let mut engine = Engine::new(
        5,
        5,
        Player::new(Civilization::English),
        vec![Player::new(Civilization::Zulu)],
    );
    engine.game.map.tile_at_mut(Location::new(1, 1)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(2, 1)).terrain = Terrain::Grassland;
    let legion = engine.game.spawn_unit(
        UnitClass::Legion,
        Location::new(1, 1),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine
        .game
        .add_city(PlayerId::new(1), "Umgungundlovu", Location::new(3, 2));
    let events = engine.submit(Command::Move {
        unit: legion,
        direction: Direction::E,
    });
    assert_eq!(
        events[0].message(),
        "English and Zulu meet for the first time"
    );
    assert!(engine.game.at_peace(PlayerId::new(0), PlayerId::new(1)));
}

#[test]
fn a_pair_only_meets_once() {
    let mut engine = Engine::new(
        5,
        5,
        Player::new(Civilization::English),
        vec![Player::new(Civilization::Zulu)],
    );
    engine.game.map.tile_at_mut(Location::new(1, 1)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(2, 1)).terrain = Terrain::Grassland;
    let legion = engine.game.spawn_unit(
        UnitClass::Legion,
        Location::new(1, 1),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine.game.spawn_unit(
        UnitClass::Militia,
        Location::new(3, 1),
        PlayerId::new(1),
        CityId::new(1),
    );
    engine.submit(Command::Move {
        unit: legion,
        direction: Direction::E,
    });
    engine.submit(Command::EndTurn);
    engine.submit(Command::EndTurn);
    let events = engine.submit(Command::Move {
        unit: legion,
        direction: Direction::W,
    });
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0].message(),
        format!("Unit {} moves W", legion.index())
    );
}

#[test]
fn a_civilization_does_not_meet_itself() {
    let mut engine = Engine::new(
        5,
        5,
        Player::new(Civilization::English),
        vec![Player::new(Civilization::Zulu)],
    );
    engine.game.map.tile_at_mut(Location::new(1, 1)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(2, 1)).terrain = Terrain::Grassland;
    let legion = engine.game.spawn_unit(
        UnitClass::Legion,
        Location::new(1, 1),
        PlayerId::new(0),
        CityId::new(0),
    );
    let events = engine.submit(Command::Move {
        unit: legion,
        direction: Direction::E,
    });
    assert_eq!(
        events[0].message(),
        format!("Unit {} moves E", legion.index())
    );
    assert_eq!(events.len(), 1);
    assert!(!engine.game.have_met(PlayerId::new(0), PlayerId::new(0)));
}

#[test]
fn a_city_grows_and_reports_it() {
    let mut engine = test_engine();
    engine.game.map.tile_at_mut(Location::new(1, 1)).terrain = Terrain::Grassland;
    engine
        .game
        .add_city(PlayerId::new(0), "London", Location::new(1, 1));
    engine.game.auto_assign_work(CityId::new(0));
    let events = engine.submit(Command::EndTurn);
    let message = events
        .iter()
        .find(|e| e.message().contains("grows"))
        .map(|e| e.message().to_string());
    assert_eq!(message, Some("London grows to size 2".to_string()));
    let city = &engine.game.cities[0];
    assert_eq!(city.population(), 2);
}

#[test]
fn a_city_produces_units() {
    let mut engine = Engine::new(7, 7, Player::new(Civilization::English), Vec::new());
    // Grassland centre plus forest ring tiles that yield resources.
    engine.game.map.tile_at_mut(Location::new(3, 3)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(2, 3)).terrain = Terrain::Forest;
    engine.game.map.tile_at_mut(Location::new(3, 2)).terrain = Terrain::Forest;
    engine.game.map.tile_at_mut(Location::new(4, 3)).terrain = Terrain::Forest;
    engine.game.map.tile_at_mut(Location::new(3, 4)).terrain = Terrain::Forest;
    engine.game.spawn_unit(
        UnitClass::Settler,
        Location::new(3, 3),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine.submit(Command::FoundCity {
        unit: UnitId::new(0),
        name: "London".to_string(),
    });
    engine.submit(Command::SetProductionTarget {
        city: CityId::new(0),
        target: ProductionTarget::Unit(UnitClass::Militia),
    });
    // Centre grassland 0 resources + 1 forest worked tile (2 resources).
    // Militia costs 10, so it should finish within 5 turns.
    let mut produced = false;
    for _ in 0..8 {
        let events = engine.submit(Command::EndTurn);
        if events
            .iter()
            .any(|e| e.message() == "London produces Militia")
        {
            produced = true;
        }
    }
    assert!(produced, "expected production event across the turns");
    assert!(
        engine
            .game
            .units
            .iter()
            .any(|u| u.unit_class == UnitClass::Militia)
    );
}

#[test]
fn production_choices_hide_improvements_the_city_already_owns() {
    let mut engine = Engine::new(6, 6, Player::new(Civilization::English), Vec::new());
    engine.game.map.tile_at_mut(Location::new(2, 2)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(4, 4)).terrain = Terrain::Grassland;
    engine.game.spawn_unit(
        UnitClass::Settler,
        Location::new(2, 2),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine.game.spawn_unit(
        UnitClass::Settler,
        Location::new(4, 4),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine.submit(Command::FoundCity {
        unit: UnitId::new(0),
        name: "London".to_string(),
    });
    engine.submit(Command::FoundCity {
        unit: UnitId::new(1),
        name: "York".to_string(),
    });
    let london = engine.game.cities[0].id();
    let york = engine.game.cities[1].id();
    let barracks = ProductionTarget::Improvement(CityImprovement::Barracks);
    // Both fresh cities may build a Barracks.
    assert!(engine.production_choices(london).contains(&barracks));
    assert!(engine.production_choices(york).contains(&barracks));
    // Once London owns one it leaves its own list...
    engine.game.cities[0].add_improvement(CityImprovement::Barracks);
    assert!(!engine.production_choices(london).contains(&barracks));
    // ...but York may still build one.
    assert!(engine.production_choices(york).contains(&barracks));
}

fn research_engine() -> Engine {
    let mut engine = Engine::new(3, 2, Player::new(Civilization::English), Vec::new());
    engine.game.map.tile_at_mut(Location::new(1, 1)).terrain = Terrain::Grassland;
    engine.game.spawn_unit(
        UnitClass::Settler,
        Location::new(1, 1),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine.submit(Command::FoundCity {
        unit: UnitId::new(0),
        name: "London".to_string(),
    });
    engine
}

#[test]
fn founding_a_city_begins_research_on_construction() {
    let engine = research_engine();
    assert_eq!(
        engine.game.advancement_in_progress(PlayerId::new(0)),
        Some(Advancement::Construction)
    );
    assert_eq!(engine.game.research_progress(PlayerId::new(0)), 0);
}

#[test]
fn set_research_target_changes_the_research_target() {
    let mut engine = research_engine();
    let events = engine.submit(Command::SetResearchTarget {
        advancement: Advancement::Wheel,
    });
    assert_eq!(
        engine.game.advancement_in_progress(PlayerId::new(0)),
        Some(Advancement::Wheel)
    );
    assert!(
        events
            .iter()
            .any(|e| e.message() == "English begin researching Wheel")
    );
}

#[test]
fn set_research_target_rejects_unmet_prerequisites() {
    let mut engine = research_engine();
    let events = engine.submit(Command::SetResearchTarget {
        advancement: Advancement::Astronomy,
    });
    assert_eq!(
        engine.game.advancement_in_progress(PlayerId::new(0)),
        Some(Advancement::Construction)
    );
    assert!(
        !engine
            .game
            .can_research(PlayerId::new(0), Advancement::Astronomy)
    );
    let error = events
        .iter()
        .find(|e| e.message() == "Cannot research Astronomy: prerequisites not met");
    assert!(error.is_some());
}

#[test]
fn set_research_target_rejects_an_already_discovered_advancement() {
    let mut engine = research_engine();
    // Discover Construction by accumulating its cost.
    for _ in 0..30 {
        engine.submit(Command::EndTurn);
    }
    assert!(engine.game.players[0].has_advancement(Advancement::Construction));
    let events = engine.submit(Command::SetResearchTarget {
        advancement: Advancement::Construction,
    });
    assert!(
        events
            .iter()
            .any(|e| e.message() == "Cannot research Construction: already discovered")
    );
}

#[test]
fn research_accumulates_and_discovers_the_advance_ment() {
    let mut engine = research_engine();
    engine.submit(Command::SetResearchTarget {
        advancement: Advancement::Wheel,
    });
    // Wheel costs 15; the city produces 4 research per turn.
    let mut discovered = false;
    for _ in 0..20 {
        let events = engine.submit(Command::EndTurn);
        if events
            .iter()
            .any(|e| e.message() == "English discover Wheel")
        {
            discovered = true;
        }
    }
    assert!(discovered, "expected the research discovery event");
    assert!(engine.game.players[0].has_advancement(Advancement::Wheel));
    assert_eq!(engine.game.advancement_in_progress(PlayerId::new(0)), None);
}

#[test]
fn a_city_starves_without_food() {
    let mut engine = Engine::new(5, 5, Player::new(Civilization::English), Vec::new());
    engine.game.map.tile_at_mut(Location::new(2, 2)).terrain = Terrain::Grassland;
    engine.game.spawn_unit(
        UnitClass::Settler,
        Location::new(2, 2),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine.submit(Command::FoundCity {
        unit: UnitId::new(0),
        name: "London".to_string(),
    });
    // Drive the city to size 5 (net food -3) so the food store drains each turn.
    for _ in 0..4 {
        engine.game.cities[0].grow();
    }
    assert_eq!(engine.game.cities[0].population(), 5);
    let events = engine.submit(Command::EndTurn);
    assert!(
        events.iter().any(|e| e.message() == "London is starving"),
        "expected starvation, got {:?}",
        events.iter().map(|e| e.message()).collect::<Vec<_>>()
    );
}

#[test]
fn setting_production_on_a_foreign_city_is_rejected() {
    let mut engine = Engine::new(
        5,
        5,
        Player::new(Civilization::English),
        vec![Player::new(Civilization::Zulu)],
    );
    engine
        .game
        .add_city(PlayerId::new(1), "Umgungundlovu", Location::new(3, 3));
    let events = engine.submit(Command::SetProductionTarget {
        city: CityId::new(0),
        target: ProductionTarget::Unit(UnitClass::Militia),
    });
    assert_eq!(events[0].message(), "No such city");
}

#[test]
fn setting_production_rejects_a_unit_requiring_an_undiscovered_advancement() {
    let mut engine = research_engine();
    let events = engine.submit(Command::SetProductionTarget {
        city: CityId::new(0),
        target: ProductionTarget::Unit(UnitClass::Knight),
    });
    assert!(
        events
            .iter()
            .any(|e| e.message() == "Cannot produce Unit(Knight): requires Chivalry")
    );
}

#[test]
fn setting_production_allows_gated_units_once_the_advancement_is_discovered() {
    let mut engine = research_engine();
    // Wheel costs 15; the city produces 4 research per turn.
    engine.submit(Command::SetResearchTarget {
        advancement: Advancement::Wheel,
    });
    for _ in 0..20 {
        engine.submit(Command::EndTurn);
    }
    assert!(engine.game.players[0].has_advancement(Advancement::Wheel));
    let events = engine.submit(Command::SetProductionTarget {
        city: CityId::new(0),
        target: ProductionTarget::Unit(UnitClass::Chariot),
    });
    assert!(
        events
            .iter()
            .any(|e| e.message() == "London begins producing Unit(Chariot)")
    );
}

#[test]
fn setting_production_allows_units_with_no_required_advancement() {
    let mut engine = research_engine();
    let events = engine.submit(Command::SetProductionTarget {
        city: CityId::new(0),
        target: ProductionTarget::Unit(UnitClass::Militia),
    });
    assert!(
        events
            .iter()
            .any(|e| e.message() == "London begins producing Unit(Militia)")
    );
}

#[test]
fn city_footprint_is_the_21_tile_ring() {
    let engine = Engine::new(7, 7, Player::new(Civilization::English), Vec::new());
    let footprint = engine.game.city_footprint(Location::new(3, 3));
    assert_eq!(footprint.len(), 21);
    assert!(footprint.contains(&Location::new(3, 3)));
    // Octant corners (distance 2,2) are excluded; ring edges included.
    assert!(footprint.contains(&Location::new(2, 3)));
    assert!(footprint.contains(&Location::new(3, 5)));
    assert!(!footprint.contains(&Location::new(1, 1)));
    assert!(!footprint.contains(&Location::new(5, 5)));
}

#[test]
fn a_city_automatically_works_its_highest_yield_tile() {
    let mut engine = Engine::new(7, 7, Player::new(Civilization::English), Vec::new());
    engine.game.map.tile_at_mut(Location::new(3, 3)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(4, 3)).terrain = Terrain::Forest;
    engine.game.map.tile_at_mut(Location::new(2, 3)).terrain = Terrain::Hills;
    engine.game.spawn_unit(
        UnitClass::Settler,
        Location::new(3, 3),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine.submit(Command::FoundCity {
        unit: UnitId::new(0),
        name: "London".to_string(),
    });
    let city = &engine.game.cities[0];
    // Size 1 works the city centre plus one ring tile; forest (2
    // resources) beats hills (1).
    assert_eq!(
        city.worked_tiles(),
        &[Location::new(3, 3), Location::new(4, 3)]
    );
    let (food, resources) = engine.game.city_income(CityId::new(0));
    assert_eq!(resources, 2);
    assert_eq!(food, 3);
}

#[test]
fn capturing_a_city_harvests_more_tiles_as_it_grows() {
    let mut engine = Engine::new(7, 7, Player::new(Civilization::English), Vec::new());
    engine.game.map.tile_at_mut(Location::new(3, 3)).terrain = Terrain::Grassland;
    engine.game.spawn_unit(
        UnitClass::Settler,
        Location::new(3, 3),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine.submit(Command::FoundCity {
        unit: UnitId::new(0),
        name: "London".to_string(),
    });
    assert_eq!(engine.game.cities[0].worked_tiles().len(), 2);
    // Grow to size 4 (engine-driven growth auto-assigns more ring tiles).
    for _ in 0..6 {
        engine.submit(Command::EndTurn);
    }
    assert_eq!(engine.game.cities[0].population(), 4);
    assert_eq!(engine.game.cities[0].worked_tiles().len(), 5);
}

/// A 4-by-3 sea with dry strips at (0,1) and (3,1) separated by the open
/// water of (1,1) and (2,1), so a trireme can take on cargo on the western
/// shore, sail the strait, and put it ashore in the east.
fn ferry_map() -> Engine {
    let mut engine = Engine::new(4, 3, english_player(), Vec::new());
    engine.game.map.tile_at_mut(Location::new(0, 1)).terrain = Terrain::Grassland;
    engine.game.map.tile_at_mut(Location::new(3, 1)).terrain = Terrain::Grassland;
    engine
}

#[test]
fn a_land_unit_boards_a_friendly_trireme_waiting_in_water() {
    let mut engine = ferry_map();
    let trireme = engine.game.spawn_unit(
        UnitClass::Trireme,
        Location::new(1, 1),
        PlayerId::new(0),
        CityId::new(0),
    );
    let knight = engine.game.spawn_unit(
        UnitClass::Knight,
        Location::new(0, 1),
        PlayerId::new(0),
        CityId::new(0),
    );
    let events = engine.submit(Command::Move {
        unit: knight,
        direction: Direction::E,
    });
    assert_eq!(
        events[0].message(),
        format!("Unit {} boards the ship", knight.index())
    );
    let boarding = engine
        .game
        .units
        .iter()
        .find(|unit| unit.id() == knight)
        .unwrap();
    assert!(boarding.is_transported());
    assert_eq!(boarding.aboard(), Some(trireme));
    assert_eq!(boarding.location, Location::new(1, 1));
    assert_eq!(boarding.moves_remaining(), 3);
    assert_eq!(boarding.order(), UnitOrder::Idle);
}

#[test]
fn every_naval_transport_carries_two_units_and_rejects_a_third() {
    for transport in [UnitClass::Trireme, UnitClass::Sail, UnitClass::Frigate] {
        let mut engine = ferry_map();
        let carrier = engine.game.spawn_unit(
            transport,
            Location::new(1, 1),
            PlayerId::new(0),
            CityId::new(0),
        );
        let first = engine.game.spawn_unit(
            UnitClass::Knight,
            Location::new(0, 1),
            PlayerId::new(0),
            CityId::new(0),
        );
        let second = engine.game.spawn_unit(
            UnitClass::Militia,
            Location::new(0, 1),
            PlayerId::new(0),
            CityId::new(0),
        );
        let third = engine.game.spawn_unit(
            UnitClass::Legion,
            Location::new(0, 1),
            PlayerId::new(0),
            CityId::new(0),
        );
        for boarder in [first, second, third] {
            engine.submit(Command::Move {
                unit: boarder,
                direction: Direction::E,
            });
        }
        let aboard = engine
            .game
            .units
            .iter()
            .filter(|unit| unit.aboard() == Some(carrier))
            .count();
        assert_eq!(aboard, 2, "{transport:?}");
        assert!(
            engine
                .game
                .units
                .iter()
                .any(|unit| unit.id() == first && unit.is_transported())
        );
        assert!(
            engine
                .game
                .units
                .iter()
                .any(|unit| unit.id() == second && unit.is_transported())
        );
        let third_unit = engine
            .game
            .units
            .iter()
            .find(|unit| unit.id() == third)
            .unwrap();
        assert!(!third_unit.is_transported());
        assert_eq!(third_unit.location, Location::new(0, 1));
        assert_eq!(third_unit.moves_remaining(), 1);
    }
}

#[test]
fn a_land_unit_cannot_board_an_enemy_trireme_across_the_water() {
    let mut engine = Engine::new(
        4,
        3,
        english_player(),
        vec![Player::new(Civilization::Zulu)],
    );
    engine.game.declare_war(PlayerId::new(0), PlayerId::new(1));
    engine.game.map.tile_at_mut(Location::new(0, 1)).terrain = Terrain::Grassland;
    engine.game.spawn_unit(
        UnitClass::Trireme,
        Location::new(1, 1),
        PlayerId::new(1),
        CityId::new(1),
    );
    let knight = engine.game.spawn_unit(
        UnitClass::Knight,
        Location::new(0, 1),
        PlayerId::new(0),
        CityId::new(0),
    );
    let events = engine.submit(Command::Move {
        unit: knight,
        direction: Direction::E,
    });
    assert_eq!(
        events[0].message(),
        format!("Unit {} cannot board: no ship there", knight.index())
    );
    let knight_unit = engine
        .game
        .units
        .iter()
        .find(|unit| unit.id() == knight)
        .unwrap();
    assert!(!knight_unit.is_transported());
    assert_eq!(knight_unit.location, Location::new(0, 1));
}

#[test]
fn a_trireme_carries_its_cargo_across_the_sea_and_back_to_land() {
    let mut engine = ferry_map();
    let trireme = engine.game.spawn_unit(
        UnitClass::Trireme,
        Location::new(1, 1),
        PlayerId::new(0),
        CityId::new(0),
    );
    let knight = engine.game.spawn_unit(
        UnitClass::Knight,
        Location::new(0, 1),
        PlayerId::new(0),
        CityId::new(0),
    );
    // Aboard on the western shore; the trireme then sails the strait.
    engine.submit(Command::Move {
        unit: knight,
        direction: Direction::E,
    });
    engine.submit(Command::Move {
        unit: trireme,
        direction: Direction::E,
    });
    let cargo = engine
        .game
        .units
        .iter()
        .find(|unit| unit.id() == knight)
        .unwrap();
    assert!(cargo.is_transported());
    assert_eq!(cargo.location, Location::new(2, 1));
    let carrier = engine
        .game
        .units
        .iter()
        .find(|unit| unit.id() == trireme)
        .unwrap();
    assert_eq!(carrier.location, Location::new(2, 1));
    // Disembark onto the eastern shore.
    let events = engine.submit(Command::Move {
        unit: knight,
        direction: Direction::E,
    });
    assert_eq!(
        events[0].message(),
        format!("Unit {} disembarks", knight.index())
    );
    let disembarked = engine
        .game
        .units
        .iter()
        .find(|unit| unit.id() == knight)
        .unwrap();
    assert!(!disembarked.is_transported());
    assert_eq!(disembarked.location, Location::new(3, 1));
    assert_eq!(disembarked.moves_remaining(), 3);
}

#[test]
fn a_transported_unit_cannot_be_instructed_while_at_sea() {
    let mut engine = ferry_map();
    engine.game.spawn_unit(
        UnitClass::Trireme,
        Location::new(1, 1),
        PlayerId::new(0),
        CityId::new(0),
    );
    let knight = engine.game.spawn_unit(
        UnitClass::Knight,
        Location::new(0, 1),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine.submit(Command::Move {
        unit: knight,
        direction: Direction::E,
    });
    // With no shore to land on it must not paddle to another patch of open
    // water, stand watch, or dig in.
    let clear_water = engine.submit(Command::Move {
        unit: knight,
        direction: Direction::N,
    });
    assert_eq!(
        clear_water[0].message(),
        format!("Unit {} cannot cross land/sea border", knight.index())
    );
    let sentry = engine.submit(Command::Sentry { unit: knight });
    assert_eq!(
        sentry[0].message(),
        format!("Unit {} is aboard a ship", knight.index())
    );
    let fortify = engine.submit(Command::Fortify { unit: knight });
    assert_eq!(
        fortify[0].message(),
        format!("Unit {} is aboard a ship", knight.index())
    );
    let cancel = engine.submit(Command::CancelOrder { unit: knight });
    assert_eq!(
        cancel[0].message(),
        format!("Unit {} is aboard a ship", knight.index())
    );
    let cargo = engine
        .game
        .units
        .iter()
        .find(|unit| unit.id() == knight)
        .unwrap();
    assert!(cargo.is_transported());
    assert_eq!(cargo.location, Location::new(1, 1));
}

#[test]
fn a_transported_settler_cannot_found_a_city_from_the_sea() {
    let mut engine = ferry_map();
    engine.game.spawn_unit(
        UnitClass::Trireme,
        Location::new(1, 1),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine.game.spawn_unit(
        UnitClass::Settler,
        Location::new(0, 1),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine.submit(Command::Move {
        unit: UnitId::new(1),
        direction: Direction::E,
    });
    let events = engine.submit(Command::FoundCity {
        unit: UnitId::new(1),
        name: "London".to_string(),
    });
    assert_eq!(events[0].message(), "Unit 1 is aboard a ship");
    assert!(engine.game.cities.is_empty());
}

#[test]
fn a_trireme_lost_at_sea_dissolves_its_cargo_which_never_defends() {
    let mut engine = Engine::new(
        4,
        3,
        english_player(),
        vec![Player::new(Civilization::Zulu)],
    );
    engine.game.declare_war(PlayerId::new(0), PlayerId::new(1));
    let trireme = engine.game.spawn_unit(
        UnitClass::Trireme,
        Location::new(2, 1),
        PlayerId::new(1),
        CityId::new(1),
    );
    let knight = engine.game.spawn_unit(
        UnitClass::Knight,
        Location::new(2, 1),
        PlayerId::new(1),
        CityId::new(1),
    );
    // Board the Zulu cargo directly; only the current player can `submit`, so
    // the English frigate below is the one that does the attacking.
    engine
        .game
        .units
        .iter_mut()
        .find(|unit| unit.id() == knight)
        .unwrap()
        .board(trireme);
    let frigate = engine.game.spawn_unit(
        UnitClass::Frigate,
        Location::new(1, 1),
        PlayerId::new(0),
        CityId::new(0),
    );
    // An enemy frigate sinks both the trireme and its cargo at once; the
    // ferried knight never fights.
    let events = engine.submit(Command::Move {
        unit: frigate,
        direction: Direction::E,
    });
    assert!(events.iter().any(|event| event.message()
        == format!("1 transported units are lost with Unit {}", trireme.index())));
    assert!(events.iter().any(|event| event.message()
        == format!("Unit {} defeats Unit {}", frigate.index(), trireme.index())));
    assert!(!engine.game.units.iter().any(|unit| unit.id() == trireme));
    assert!(!engine.game.units.iter().any(|unit| unit.id() == knight));
}

#[test]
fn a_transported_unit_has_no_map_square_of_its_own() {
    let mut engine = ferry_map();
    let trireme = engine.game.spawn_unit(
        UnitClass::Trireme,
        Location::new(1, 1),
        PlayerId::new(0),
        CityId::new(0),
    );
    let knight = engine.game.spawn_unit(
        UnitClass::Knight,
        Location::new(0, 1),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine.submit(Command::Move {
        unit: knight,
        direction: Direction::E,
    });
    // The trireme stands alone on its tile for the map's purposes, but the
    // ferried unit remains addressable when selected.
    assert_eq!(engine.units_at(1, 1).len(), 1);
    assert_eq!(engine.units_at(1, 1)[0].id(), trireme);
    assert_eq!(
        engine.unit(knight).map(|unit| unit.aboard()),
        Some(Some(trireme))
    );
}
