use crate::game_engine::{Command, Event, GameView, Player};
use crate::model::cartography::{Direction, Location, Tile};
use crate::model::cities::City;
use crate::model::civilizations::{Civilization, PlayerId};
use crate::model::geography::{Geography, GeographyImprovement};
use crate::model::units::{Unit, UnitId};

use super::game::Game;
use crate::game_engine::{MoveError, Rng, SettleError};

const DEFAULT_SEED: u64 = 0xC0FFEE;
const HIT_POINTS: u32 = 10;

pub struct Engine {
    game: Game,
    turn: u32,
    current_player_index: PlayerId,
    events: Vec<Event>,
    rng: Rng,
}

impl Engine {
    pub fn new(width: usize, height: usize, first: Player, rest: Vec<Player>) -> Self {
        Engine::with_seed(width, height, first, rest, DEFAULT_SEED)
    }

    pub fn with_seed(
        width: usize,
        height: usize,
        first: Player,
        rest: Vec<Player>,
        seed: u64,
    ) -> Self {
        Engine {
            game: Game::new(width, height, first, rest),
            turn: 1,
            current_player_index: PlayerId::new(0),
            events: Vec::new(),
            rng: Rng::new(seed),
        }
    }

    pub fn submit(&mut self, command: Command) -> Vec<Event> {
        match command {
            Command::Move { unit, direction } => self.move_unit(unit, direction),
            Command::Fortify { unit } => self.fortify(unit),
            Command::Sentry { unit } => self.sentry(unit),
            Command::Work { unit, improvement } => self.work(unit, improvement),
            Command::CancelOrder { unit } => self.cancel_order(unit),
            Command::FoundCity { unit, name } => self.found_city(unit, name),
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

        let owner = self.owned_unit(unit).unwrap().owner();
        let enemies_present = self
            .game
            .units
            .iter()
            .any(|u| u.location == destination && u.owner() != owner);
        if enemies_present {
            self.resolve_move_combat(unit, destination);
            return;
        }

        let mut_unit = self.owned_unit_mut(unit).unwrap();
        mut_unit.location = destination;
        mut_unit.spend_moves(cost);
        self.game.reveal_tiles_at(owner, destination);
        self.events.push(Event::new(format!(
            "Unit {} moves {:?}",
            unit.index(),
            direction
        )));
    }

    fn resolve_move_combat(&mut self, attacker: UnitId, tile: Location) {
        let attacker_idx = self
            .game
            .units
            .iter()
            .position(|u| u.id() == attacker)
            .expect("the moving unit exists");
        let defender_idx = self.select_defender(attacker_idx, tile);

        let attacker_id = self.game.units[attacker_idx].id();
        let defender_id = self.game.units[defender_idx].id();
        self.events.push(Event::new(format!(
            "Unit {} attacks Unit {}",
            attacker_id.index(),
            defender_id.index()
        )));

        let attacker_power = self.attacker_power(&self.game.units[attacker_idx]);
        let defender_power = self.defender_power(&self.game.units[defender_idx]);
        let attacker_won = self.resolve_combat(attacker_power, defender_power);

        if attacker_won {
            let owner = self.game.units[attacker_idx].owner();
            let was_veteran = self.game.units[attacker_idx].is_veteran();
            self.game.remove_unit(defender_id);
            let tile_is_clear = !self
                .game
                .units
                .iter()
                .any(|unit| unit.location == tile && unit.owner() != owner);
            let attacker_unit = self.owned_unit_mut(attacker_id).unwrap();
            if tile_is_clear {
                attacker_unit.location = tile;
            }
            attacker_unit.spend_turn();
            if !was_veteran {
                attacker_unit.promote();
            }
            self.events.push(Event::new(format!(
                "Unit {} defeats Unit {}",
                attacker_id.index(),
                defender_id.index()
            )));
        } else {
            self.game.remove_unit(attacker_id);
            self.events.push(Event::new(format!(
                "Unit {} repels Unit {}",
                defender_id.index(),
                attacker_id.index()
            )));
        }
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

    fn found_city(&mut self, unit: UnitId, name: String) {
        let (owner, location) = match self.ensure_can_found(unit) {
            Ok(legal) => legal,
            Err(SettleError::NoSuchUnit(_)) => {
                self.events.push(Event::new("No such unit"));
                return;
            }
            Err(error) => {
                self.events.push(Event::new(error.message()));
                return;
            }
        };
        self.game.remove_unit(unit);
        self.game.add_city(owner, name.clone(), location);
        self.game.reveal_tiles_surrounding_city_at(owner, location);
        self.events
            .push(Event::new(format!("Unit {} founds {}", unit.index(), name)));
    }

    fn ensure_can_found(&self, unit: UnitId) -> Result<(PlayerId, Location), SettleError> {
        let unit = self.owned_unit(unit).ok_or(SettleError::NoSuchUnit(unit))?;
        if !unit.unit_class.can_found_city() {
            return Err(SettleError::NotASettler(unit.id()));
        }
        let location = unit.location;
        if self.game.map.tile_at(location).geography.is_water() {
            return Err(SettleError::LandRequired(unit.id()));
        }
        if self
            .game
            .cities
            .iter()
            .any(|city| city.location == location)
        {
            return Err(SettleError::CityAlreadyHere(location));
        }
        Ok((unit.owner(), location))
    }

    fn select_defender(&self, attacker_idx: usize, tile: Location) -> usize {
        let owner = self.game.units[attacker_idx].owner();
        self.game
            .units
            .iter()
            .enumerate()
            .filter(|(_, unit)| unit.location == tile && unit.owner() != owner)
            .max_by(|(_, a), (_, b)| {
                (self.defender_power(a), a.id().index())
                    .cmp(&(self.defender_power(b), b.id().index()))
            })
            .expect("the target's tile always holds at least the named enemy unit")
            .0
    }

    fn attacker_power(&self, unit: &Unit) -> u32 {
        let base = unit.unit_class.attack() as u32 * 10;
        if unit.is_veteran() {
            base * 3 / 2
        } else {
            base
        }
    }

    fn defender_power(&self, unit: &Unit) -> u32 {
        let base = unit.unit_class.defence() as u32 * 10;
        let mut power = base;
        if self.game.map.tile_at(unit.location).geography == Geography::Mountain {
            power *= 2;
        }
        let is_in_home_city = self
            .game
            .cities
            .iter()
            .any(|city| city.location == unit.location && city.owner() == unit.owner());
        if is_in_home_city {
            power = power * 3 / 2;
        }
        if unit.is_veteran() {
            power = power * 3 / 2;
        }
        power
    }

    fn resolve_combat(&mut self, attacker_power: u32, defender_power: u32) -> bool {
        let total = attacker_power + defender_power;
        if total == 0 {
            return false;
        }
        let mut attacker_hp = HIT_POINTS;
        let mut defender_hp = HIT_POINTS;
        while attacker_hp > 0 && defender_hp > 0 {
            let hit = self.rng.in_range(total);
            if hit < attacker_power {
                defender_hp -= 1;
            } else {
                attacker_hp -= 1;
            }
        }
        defender_hp == 0
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
        self.game.players[self.current_player_index.index()].explored_at(x, y)
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

    #[test]
    fn a_settler_founds_a_city() {
        let mut engine = Engine::new(5, 5, Player::new(Civilization::English), Vec::new());
        engine.game.map.tile_at_mut(Location::new(2, 2)).geography = Geography::Grassland;
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
        assert_eq!(events[0].message(), "Unit 0 founds London");
        assert!(engine.game.units.is_empty());
        assert_eq!(engine.game.cities.len(), 1);
        assert_eq!(engine.game.cities[0].name, "London");
        assert_eq!(engine.game.cities[0].location, Location::new(2, 2));
        assert_eq!(engine.game.cities[0].owner(), PlayerId::new(0));
    }

    #[test]
    fn founding_a_city_reveals_tiles_around_it_for_its_owner() {
        let mut engine = two_player_engine();
        engine.game.map.tile_at_mut(Location::new(1, 1)).geography = Geography::Grassland;
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
        engine.game.map.tile_at_mut(Location::new(2, 2)).geography = Geography::Grassland;
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
        engine.game.map.tile_at_mut(Location::new(2, 2)).geography = Geography::Grassland;
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
        Engine::with_seed(
            5,
            5,
            Player::new(Civilization::English),
            vec![Player::new(Civilization::Zulu)],
            11,
        )
    }

    #[test]
    fn attacker_defeats_the_target_and_takes_its_tile() {
        let mut engine = blank_war_map();
        engine.game.map.tile_at_mut(Location::new(2, 2)).geography = Geography::Grassland;
        engine.game.map.tile_at_mut(Location::new(3, 2)).geography = Geography::Grassland;
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
    fn the_strongest_defender_on_the_target_tile_absorbs_the_attack() {
        let mut engine = Engine::with_seed(
            5,
            5,
            Player::new(Civilization::English),
            vec![Player::new(Civilization::Zulu)],
            1,
        );
        engine.game.map.tile_at_mut(Location::new(2, 2)).geography = Geography::Grassland;
        engine.game.map.tile_at_mut(Location::new(3, 2)).geography = Geography::Grassland;
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
        engine.game.map.tile_at_mut(Location::new(2, 2)).geography = Geography::Grassland;
        engine.game.map.tile_at_mut(Location::new(3, 2)).geography = Geography::Grassland;
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
        engine.game.map.tile_at_mut(Location::new(2, 2)).geography = Geography::Grassland;
        engine.game.map.tile_at_mut(Location::new(2, 3)).geography = Geography::Grassland;
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
        engine.game.map.tile_at_mut(Location::new(2, 2)).geography = Geography::Grassland;
        engine.game.map.tile_at_mut(Location::new(3, 2)).geography = Geography::Grassland;
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
        engine.game.map.tile_at_mut(Location::new(2, 2)).geography = Geography::Grassland;
        engine.game.map.tile_at_mut(Location::new(2, 3)).geography = Geography::Mountain;
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
}
