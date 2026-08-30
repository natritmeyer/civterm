use crate::game_engine::{Command, Event, GameView, Player};
use crate::model::cartography::{Direction, Location, Tile};
use crate::model::cities::City;
use crate::model::civilizations::{Civilization, PlayerId};
use crate::model::geography::GeographyImprovement;
use crate::model::units::{Unit, UnitId};

use super::game::Game;
use crate::game_engine::MoveError;

pub struct Engine {
    game: Game,
    turn: u32,
    current_player_index: PlayerId,
    events: Vec<Event>,
}

impl Engine {
    pub fn new(width: usize, height: usize, first: Player, rest: Vec<Player>) -> Self {
        Engine {
            game: Game::new(width, height, first, rest),
            turn: 1,
            current_player_index: PlayerId::new(0),
            events: Vec::new(),
        }
    }

    pub fn submit(&mut self, command: Command) -> Vec<Event> {
        match command {
            Command::Move { unit, direction } => self.move_unit(unit, direction),
            Command::Fortify { unit } => self.fortify(unit),
            Command::Sentry { unit } => self.sentry(unit),
            Command::Work { unit, improvement } => self.work(unit, improvement),
            Command::CancelOrder { unit } => self.cancel_order(unit),
            Command::EndTurn => self.end_turn(),
        }
        std::mem::take(&mut self.events)
    }

    fn move_unit(&mut self, unit: UnitId, direction: Direction) {
        let (destination, cost) = match self.ensure_can_move(unit, direction) {
            Ok(legal) => legal,
            Err(MoveError::NoSuchUnit(_)) => {
                self.events.push(Event::new("No such unit"));
                return;
            }
            Err(error) => {
                self.events.push(Event::new(error.message()));
                return;
            }
        };

        let mut_unit = self.owned_unit_mut(unit).unwrap();
        mut_unit.location = destination;
        mut_unit.spend_moves(cost);
        let owner = mut_unit.owner();
        self.game.reveal(owner, destination);
        self.events.push(Event::new(format!(
            "Unit {} moves {:?}",
            unit.index(),
            direction
        )));
    }

    fn ensure_medium_access(&self, unit: &Unit, destination: Location) -> Result<(), MoveError> {
        let tile_is_water = self.game.map.tile_at(destination).geography.is_water();
        if unit.unit_class.can_travel_water() != tile_is_water {
            Err(MoveError::CannotCrossLandSeaBorder(unit.id()))
        } else {
            Ok(())
        }
    }

    fn ensure_can_move(
        &self,
        unit: UnitId,
        direction: Direction,
    ) -> Result<(Location, u8), MoveError> {
        let unit = self.ensure_unit_owned(unit)?;
        self.ensure_moves_remaining(unit)?;
        let destination = self.ensure_destination_on_map(unit.location, direction)?;
        self.ensure_medium_access(unit, destination)?;
        let cost = self.ensure_affordable(unit, destination)?;
        Ok((destination, cost))
    }

    fn ensure_unit_owned(&self, unit: UnitId) -> Result<&Unit, MoveError> {
        self.owned_unit(unit).ok_or(MoveError::NoSuchUnit(unit))
    }

    fn ensure_moves_remaining(&self, unit: &Unit) -> Result<(), MoveError> {
        if unit.moves_remaining() > 0 {
            Ok(())
        } else {
            Err(MoveError::NoMovesRemaining(unit.id()))
        }
    }

    fn ensure_destination_on_map(
        &self,
        from: Location,
        direction: Direction,
    ) -> Result<Location, MoveError> {
        self.game
            .map
            .destination(from, direction)
            .ok_or(MoveError::CannotMoveThere)
    }

    fn ensure_affordable(&self, unit: &Unit, destination: Location) -> Result<u8, MoveError> {
        let cost = self.game.map.tile_at(destination).geography.movement_cost();
        if unit.moves_remaining() >= cost {
            Ok(cost)
        } else {
            Err(MoveError::TerrainTooDifficult(unit.id()))
        }
    }

    fn fortify(&mut self, unit: UnitId) {
        match self.owned_unit_mut(unit) {
            Some(u) => {
                u.fortify();
                u.spend_turn();
                self.events
                    .push(Event::new(format!("Unit {} fortifies", unit.index())));
            }
            None => self.events.push(Event::new("No such unit")),
        }
    }

    fn sentry(&mut self, unit: UnitId) {
        match self.owned_unit_mut(unit) {
            Some(u) => {
                u.sentry();
                u.spend_turn();
                self.events
                    .push(Event::new(format!("Unit {} stands sentry", unit.index())));
            }
            None => self.events.push(Event::new("No such unit")),
        }
    }

    fn work(&mut self, unit: UnitId, improvement: GeographyImprovement) {
        match self.owned_unit_mut(unit) {
            Some(u) => {
                u.work(improvement);
                u.spend_turn();
                self.events.push(Event::new(format!(
                    "Unit {} begins {:?}",
                    unit.index(),
                    improvement
                )));
            }
            None => self.events.push(Event::new("No such unit")),
        }
    }

    fn cancel_order(&mut self, unit: UnitId) {
        match self.owned_unit_mut(unit) {
            Some(u) => {
                u.cancel_order();
                u.spend_turn();
                self.events
                    .push(Event::new(format!("Unit {} order cancelled", unit.index())));
            }
            None => self.events.push(Event::new("No such unit")),
        }
    }

    fn end_turn(&mut self) {
        self.advance_to_next_player();
        self.begin_turn();
        if self.current_player_index == PlayerId::new(0) {
            self.turn += 1;
        }
        self.events.push(Event::new(format!(
            "{:?} begins turn {}",
            self.current_player(),
            self.turn
        )));
    }

    fn advance_to_next_player(&mut self) {
        let count = self.game.players.len();
        self.current_player_index = PlayerId::new((self.current_player_index.index() + 1) % count);
    }

    fn begin_turn(&mut self) {
        for unit in self
            .game
            .units
            .iter_mut()
            .filter(|unit| unit.owner() == self.current_player_index)
        {
            unit.restore_moves();
        }
    }

    fn owned_unit(&self, unit: UnitId) -> Option<&Unit> {
        self.game
            .units
            .iter()
            .find(|u| u.id() == unit && u.owner() == self.current_player_index)
    }

    fn owned_unit_mut(&mut self, unit: UnitId) -> Option<&mut Unit> {
        let unit_ref = self.game.units.iter_mut().find(|u| u.id() == unit)?;
        if unit_ref.owner() == self.current_player_index {
            Some(unit_ref)
        } else {
            None
        }
    }
}

impl GameView for Engine {
    fn width(&self) -> usize {
        self.game.map.width
    }

    fn height(&self) -> usize {
        self.game.map.height
    }

    fn tile(&self, x: usize, y: usize) -> &Tile {
        self.game.map.tile_at(Location::new(x as u16, y as u16))
    }

    fn units_at(&self, x: usize, y: usize) -> Vec<&Unit> {
        self.game
            .units
            .iter()
            .filter(|unit| unit.location.x == x as u16 && unit.location.y == y as u16)
            .collect()
    }

    fn city_at(&self, x: usize, y: usize) -> Option<&City> {
        self.game
            .cities
            .iter()
            .find(|city| city.location.x == x as u16 && city.location.y == y as u16)
    }

    fn explored(&self, x: usize, y: usize) -> bool {
        self.game.explored[self.current_player_index.index()].marks(x, y)
    }

    fn current_player(&self) -> Civilization {
        self.game.players[self.current_player_index.index()].civilization
    }

    fn turn(&self) -> u32 {
        self.turn
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::cities::CityId;
    use crate::model::geography::Geography;
    use crate::model::units::{UnitClass, UnitId, UnitOrder};

    fn english_player() -> Player {
        Player::new(Civilization::English)
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
        assert_eq!(engine.tile(0, 0).geography, Geography::Ocean);
        assert_eq!(engine.units_at(1, 1).len(), 1);
        assert!(engine.units_at(0, 0).is_empty());
    }

    #[test]
    fn move_command_moves_the_unit_within_the_map() {
        let mut engine = test_engine();
        engine.game.map.tile_at_mut(Location::new(2, 1)).geography = Geography::Grassland;
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
        engine.game.map.tile_at_mut(Location::new(2, 1)).geography = Geography::Grassland;
        engine.game.map.tile_at_mut(Location::new(0, 1)).geography = Geography::Grassland;
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
        engine.game.map.tile_at_mut(Location::new(0, 1)).geography = Geography::Grassland;
        engine.game.map.tile_at_mut(Location::new(2, 1)).geography = Geography::Grassland;
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
    fn work_command_orders_the_unit() {
        let mut engine = test_engine();
        engine.submit(Command::Work {
            unit: UnitId::new(0),
            improvement: GeographyImprovement::Road,
        });
        assert_eq!(
            engine.game.units[0].order(),
            UnitOrder::Improving(GeographyImprovement::Road)
        );
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
        engine.game.map.tile_at_mut(Location::new(1, 0)).geography = Geography::Grassland;
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
        engine.game.map.tile_at_mut(Location::new(2, 0)).geography = Geography::Grassland;
        engine.submit(Command::Move {
            unit: UnitId::new(1),
            direction: Direction::E,
        });
        assert!(engine.game.explored[0].marks(2, 0));
        assert!(!engine.game.explored[1].marks(2, 0));
    }

    #[test]
    fn end_turn_advances_the_turn_number() {
        let mut engine = test_engine();
        let events = engine.submit(Command::EndTurn);
        assert_eq!(engine.turn(), 2);
        assert_eq!(events[0].message(), "English begins turn 2");
    }

    #[test]
    fn end_turn_moves_play_to_the_next_player_without_advancing_the_turn() {
        let mut engine = two_player_engine();
        let events = engine.submit(Command::EndTurn);
        assert_eq!(engine.current_player(), Civilization::Zulu);
        assert_eq!(engine.turn(), 1);
        assert_eq!(events[0].message(), "Zulu begins turn 1");
    }

    #[test]
    fn a_unit_can_move_only_once_per_turn() {
        let mut engine = test_engine();
        engine.game.map.tile_at_mut(Location::new(2, 1)).geography = Geography::Grassland;
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
    fn difficult_terrain_costs_more_moves_points() {
        let mut engine = Engine::new(5, 1, Player::new(Civilization::English), Vec::new());
        engine.game.spawn_unit(
            UnitClass::Cavalry,
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
        );
        engine.game.map.tile_at_mut(Location::new(0, 0)).geography = Geography::Grassland;
        engine.game.map.tile_at_mut(Location::new(1, 0)).geography = Geography::Forest;
        engine.game.map.tile_at_mut(Location::new(2, 0)).geography = Geography::Forest;
        engine.submit(Command::Move {
            unit: UnitId::new(0),
            direction: Direction::E,
        });
        assert_eq!(engine.game.units[0].location, Location::new(1, 0));
        assert_eq!(engine.game.units[0].moves_remaining(), 1);
        let events = engine.submit(Command::Move {
            unit: UnitId::new(0),
            direction: Direction::E,
        });
        assert_eq!(events[0].message(), "Terrain too difficult");
        assert_eq!(engine.game.units[0].location, Location::new(1, 0));
        engine.submit(Command::Move {
            unit: UnitId::new(0),
            direction: Direction::W,
        });
        assert_eq!(engine.game.units[0].location, Location::new(0, 0));
        assert_eq!(engine.game.units[0].moves_remaining(), 0);
    }

    #[test]
    fn a_unit_without_enough_moves_cannot_enter_difficult_terrain() {
        let mut engine = Engine::new(5, 1, Player::new(Civilization::English), Vec::new());
        engine.game.spawn_unit(
            UnitClass::Settler,
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
        );
        engine.game.map.tile_at_mut(Location::new(1, 0)).geography = Geography::Forest;
        let events = engine.submit(Command::Move {
            unit: UnitId::new(0),
            direction: Direction::E,
        });
        assert_eq!(events[0].message(), "Terrain too difficult");
        assert_eq!(engine.game.units[0].location, Location::new(0, 0));
        assert_eq!(engine.game.units[0].moves_remaining(), 1);
    }

    #[test]
    fn moves_are_restored_at_the_beginning_of_the_owners_turn() {
        let mut engine = test_engine();
        engine.game.map.tile_at_mut(Location::new(2, 1)).geography = Geography::Grassland;
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
        engine.game.map.tile_at_mut(Location::new(2, 1)).geography = Geography::Grassland;
        engine.submit(Command::Move {
            unit: UnitId::new(0),
            direction: Direction::E,
        });
        assert_eq!(engine.game.units[0].moves_remaining(), 0);
        engine.submit(Command::EndTurn);
        assert_eq!(engine.current_player(), Civilization::Zulu);
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
        assert_eq!(engine.game.units[0].moves_remaining(), 0);
    }

    #[test]
    fn a_land_unit_cannot_enter_water() {
        let mut engine = test_engine();
        let events = engine.submit(Command::Move {
            unit: UnitId::new(0),
            direction: Direction::E,
        });
        assert_eq!(events[0].message(), "Unit 0 cannot cross land/sea border");
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
        engine.game.map.tile_at_mut(Location::new(2, 1)).geography = Geography::Grassland;
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
        assert_eq!(engine.turn(), 2);
        assert_eq!(events[0].message(), "English begins turn 2");
    }

    #[test]
    fn three_players_rotate_through_three_full_turns() {
        let mut engine = three_player_engine();
        let mut messages = Vec::new();
        for _ in 0..9 {
            let events = engine.submit(Command::EndTurn);
            messages.push(events[0].message().to_string());
            if messages.len() % 3 == 0 {
                assert_eq!(engine.current_player(), Civilization::English);
            }
        }
        assert_eq!(engine.turn(), 4);
        assert_eq!(
            messages,
            vec![
                "Zulu begins turn 1",
                "Roman begins turn 1",
                "English begins turn 2",
                "Zulu begins turn 2",
                "Roman begins turn 2",
                "English begins turn 3",
                "Zulu begins turn 3",
                "Roman begins turn 3",
                "English begins turn 4",
            ]
        );
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
}
