use crate::game_engine::{Command, Event, GameView, Player};
use crate::model::cartography::{Direction, Location, Tile};
use crate::model::cities::City;
use crate::model::civilizations::{Civilization, PlayerId};
use crate::model::geography::GeographyImprovement;
use crate::model::units::{Unit, UnitId};

use super::game::Game;

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
        let (width, height) = (self.game.map.width as isize, self.game.map.height as isize);
        let (dx, dy) = direction.delta();
        match self.owned_unit_mut(unit) {
            Some(u) => {
                let x = (u.location.x as isize + dx).rem_euclid(width);
                let y = u.location.y as isize + dy;
                if y >= 0 && y < height {
                    u.location = Location::new(x as u16, y as u16);
                    self.events.push(Event::new(format!(
                        "Unit {} moves {:?}",
                        unit.index(),
                        direction
                    )));
                } else {
                    self.events.push(Event::new("Cannot move there"));
                }
            }
            None => self.events.push(Event::new("No such unit")),
        }
    }

    fn fortify(&mut self, unit: UnitId) {
        match self.owned_unit_mut(unit) {
            Some(u) => {
                u.fortify();
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
                self.events
                    .push(Event::new(format!("Unit {} order cancelled", unit.index())));
            }
            None => self.events.push(Event::new("No such unit")),
        }
    }

    fn end_turn(&mut self) {
        self.advance_to_next_player();
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
        Engine::new(
            3,
            2,
            Player::new(Civilization::English),
            vec![Player::new(Civilization::Zulu)],
        )
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
        engine.submit(Command::Move {
            unit: UnitId::new(0),
            direction: Direction::E,
        });
        engine.submit(Command::Move {
            unit: UnitId::new(0),
            direction: Direction::E,
        });
        assert_eq!(engine.game.units[0].location, Location::new(0, 1));
    }

    #[test]
    fn moving_west_off_the_map_wraps_around_to_the_east() {
        let mut engine = test_engine();
        engine.submit(Command::Move {
            unit: UnitId::new(0),
            direction: Direction::W,
        });
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
        let mut engine = test_engine();
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
    fn submitting_to_an_unknown_unit_reports_an_event_without_changing_state() {
        let mut engine = test_engine();
        let events = engine.submit(Command::Fortify {
            unit: UnitId::new(99),
        });
        assert_eq!(events[0].message(), "No such unit");
        assert_eq!(engine.game.units[0].order(), UnitOrder::Idle);
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
