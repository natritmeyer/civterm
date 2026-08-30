use crate::game_engine::player::Player;
use crate::model::cartography::{Location, Map};
use crate::model::cities::{City, CityId};
use crate::model::civilizations::PlayerId;
use crate::model::units::{Unit, UnitClass, UnitId};

pub struct Game {
    pub map: Map,
    pub players: Vec<Player>,
    pub units: Vec<Unit>,
    pub cities: Vec<City>,
    next_unit_id: usize,
    next_city_id: usize,
}

impl Game {
    pub const DISCOVERY_RADIUS: u8 = 1;

    pub fn new(width: usize, height: usize, first: Player, rest: Vec<Player>) -> Self {
        let mut players = Vec::with_capacity(rest.len() + 1);
        players.push(first);
        players.extend(rest);
        for player in &mut players {
            player.seed_exploration(width, height);
        }
        Game {
            map: Map::new(width, height),
            players,
            units: Vec::new(),
            cities: Vec::new(),
            next_unit_id: 0,
            next_city_id: 0,
        }
    }

    pub fn reveal(&mut self, player: PlayerId, location: Location) {
        self.players[player.index()].reveal(location, Self::DISCOVERY_RADIUS);
    }

    pub fn spawn_unit(
        &mut self,
        unit_class: UnitClass,
        location: Location,
        owner: PlayerId,
        home_city: CityId,
    ) -> UnitId {
        let id = UnitId::new(self.next_unit_id);
        self.next_unit_id += 1;
        self.units
            .push(Unit::new(unit_class, location, owner, home_city, id));
        self.reveal(owner, location);
        id
    }

    pub fn remove_unit(&mut self, id: UnitId) -> Option<Unit> {
        let index = self.units.iter().position(|unit| unit.id() == id)?;
        Some(self.units.swap_remove(index))
    }

    pub fn add_city(
        &mut self,
        owner: PlayerId,
        name: impl Into<String>,
        location: Location,
    ) -> CityId {
        let id = CityId::new(self.next_city_id);
        self.next_city_id += 1;
        self.cities.push(City::new(name, location, owner, id));
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::civilizations::Civilization;

    fn player() -> PlayerId {
        PlayerId::new(0)
    }

    fn home() -> CityId {
        CityId::new(0)
    }

    #[test]
    fn game_created_with_the_given_players() {
        let game = Game::new(
            3,
            2,
            Player::new(Civilization::English),
            vec![Player::new(Civilization::Zulu)],
        );
        assert_eq!(game.players.len(), 2);
        assert_eq!(game.players[0].civilization, Civilization::English);
        assert_eq!(game.players[1].civilization, Civilization::Zulu);
    }

    #[test]
    fn game_can_have_a_single_player() {
        let game = Game::new(3, 2, Player::new(Civilization::Roman), Vec::new());
        assert_eq!(game.players.len(), 1);
        assert_eq!(game.players[0].civilization, Civilization::Roman);
    }

    #[test]
    fn each_player_is_seeded_with_a_world_sized_exploration() {
        let game = Game::new(
            3,
            2,
            Player::new(Civilization::English),
            vec![Player::new(Civilization::Zulu)],
        );
        let mut first = Player::new(Civilization::English);
        first.seed_exploration(3, 2);
        let mut second = Player::new(Civilization::Zulu);
        second.seed_exploration(3, 2);
        assert_eq!(game.players, vec![first, second]);
    }

    #[test]
    fn game_map_has_requested_dimensions() {
        let game = Game::new(3, 2, Player::new(Civilization::English), Vec::new());
        assert_eq!(game.map.width, 3);
        assert_eq!(game.map.height, 2);
    }

    #[test]
    fn game_starts_with_no_units() {
        let game = Game::new(3, 2, Player::new(Civilization::English), Vec::new());
        assert!(game.units.is_empty());
    }

    #[test]
    fn game_starts_with_no_cities() {
        let game = Game::new(3, 2, Player::new(Civilization::English), Vec::new());
        assert!(game.cities.is_empty());
    }

    #[test]
    fn add_city_allocates_a_city_id_and_records_its_owner() {
        let mut game = Game::new(3, 2, Player::new(Civilization::English), Vec::new());
        let london = game.add_city(PlayerId::new(0), "London", Location::new(1, 0));
        let york = game.add_city(PlayerId::new(0), "York", Location::new(0, 1));
        assert_eq!(london, CityId::new(0));
        assert_eq!(york, CityId::new(1));
        assert_eq!(game.cities.len(), 2);
        assert_eq!(game.cities[0].name, "London");
        assert_eq!(game.cities[0].owner(), PlayerId::new(0));
        assert_eq!(game.cities[1].name, "York");
    }

    #[test]
    fn spawn_unit_allocates_ids_that_resolve_in_the_list() {
        let mut game = Game::new(3, 2, Player::new(Civilization::English), Vec::new());
        let first = game.spawn_unit(UnitClass::Settler, Location::new(0, 0), player(), home());
        let second = game.spawn_unit(UnitClass::Legion, Location::new(1, 0), player(), home());
        assert_eq!(first, UnitId::new(0));
        assert_eq!(second, UnitId::new(1));
        assert_eq!(game.units.len(), 2);
        assert_eq!(game.units[0].id(), first);
        assert_eq!(game.units[1].id(), second);
        assert_eq!(game.units[0].unit_class, UnitClass::Settler);
        assert_eq!(game.units[1].unit_class, UnitClass::Legion);
    }

    #[test]
    fn removing_a_unit_does_not_disturb_other_ids() {
        let mut game = Game::new(3, 2, Player::new(Civilization::English), Vec::new());
        let first = game.spawn_unit(UnitClass::Legion, Location::new(0, 0), player(), home());
        let middle = game.spawn_unit(UnitClass::Legion, Location::new(1, 0), player(), home());
        let third = game.spawn_unit(UnitClass::Diplomat, Location::new(2, 0), player(), home());
        assert_eq!(
            game.remove_unit(middle).unwrap().unit_class,
            UnitClass::Legion
        );
        assert_eq!(game.units.len(), 2);
        assert!(game.units.iter().any(|u| u.id() == first));
        assert!(game.units.iter().any(|u| u.id() == third));
        assert!(!game.units.iter().any(|u| u.id() == middle));
    }

    #[test]
    fn removing_an_unknown_unit_reports_nothing() {
        let mut game = Game::new(3, 2, Player::new(Civilization::English), Vec::new());
        assert_eq!(game.remove_unit(UnitId::new(5)), None);
    }

    #[test]
    fn spawning_a_unit_reveals_the_area_around_it_for_its_owner() {
        let mut game = Game::new(
            5,
            5,
            Player::new(Civilization::English),
            vec![Player::new(Civilization::Zulu)],
        );
        game.spawn_unit(
            UnitClass::Settler,
            Location::new(2, 2),
            PlayerId::new(0),
            CityId::new(0),
        );
        assert!(game.players[0].explored_at(1, 1));
        assert!(game.players[0].explored_at(2, 2));
        assert!(game.players[0].explored_at(3, 3));
        assert!(!game.players[0].explored_at(4, 4));
        assert!(!game.players[1].explored_at(2, 2));
    }
}
