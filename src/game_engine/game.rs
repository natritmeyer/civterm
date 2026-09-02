use crate::game_engine::player::Player;
use crate::model::advancements::Advancement;
use crate::model::cartography::{Location, Map};
use crate::model::cities::{City, CityId, ProductionTarget};
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

    pub fn at_war(&self, a: PlayerId, b: PlayerId) -> bool {
        self.players[a.index()].at_war_with.contains(&b)
            || self.players[b.index()].at_war_with.contains(&a)
    }

    pub fn at_peace(&self, a: PlayerId, b: PlayerId) -> bool {
        self.players[a.index()].at_peace_with.contains(&b)
            || self.players[b.index()].at_peace_with.contains(&a)
    }

    pub fn met(&self, a: PlayerId, b: PlayerId) -> bool {
        self.at_war(a, b) || self.at_peace(a, b)
    }

    pub fn declare_war(&mut self, a: PlayerId, b: PlayerId) {
        if a == b {
            return;
        }
        self.players[a.index()].enter_war_with(b);
        self.players[b.index()].enter_war_with(a);
    }

    pub fn make_peace(&mut self, a: PlayerId, b: PlayerId) {
        if a == b {
            return;
        }
        self.players[a.index()].enter_peace_with(b);
        self.players[b.index()].enter_peace_with(a);
    }

    pub fn reveal_tiles_at(&mut self, player: PlayerId, location: Location) {
        self.players[player.index()].reveal_tiles_at(location, Self::DISCOVERY_RADIUS);
    }

    pub fn reveal_tiles_surrounding_city_at(&mut self, player: PlayerId, location: Location) {
        self.players[player.index()].reveal_tiles_surrounding_city_at(location);
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
        self.reveal_tiles_at(owner, location);
        id
    }

    pub fn remove_unit(&mut self, id: UnitId) -> Option<Unit> {
        let index = self.units.iter().position(|unit| unit.id() == id)?;
        Some(self.units.swap_remove(index))
    }

    /// Dissolve every unit whose home city is `city_id` (used when a city is
    /// captured). Returns the number of units disbanded.
    pub fn disband_units_homed_to(&mut self, city_id: CityId) -> u32 {
        let mut disbanded = 0;
        let mut i = 0;
        while i < self.units.len() {
            if self.units[i].home_city() == city_id {
                self.units.swap_remove(i);
                disbanded += 1;
            } else {
                i += 1;
            }
        }
        disbanded
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

    pub fn owned_units(&self, owner: PlayerId) -> Vec<UnitId> {
        self.units
            .iter()
            .filter(|unit| unit.owner() == owner)
            .map(|unit| unit.id())
            .collect()
    }

    /// Sum food and resource income a city harvests from its worked tiles
    /// (the city centre is always worked, plus each chosen ring tile).
    pub fn city_income(&self, city_id: CityId) -> (u32, u32) {
        let city = self
            .cities
            .iter()
            .find(|city| city.id() == city_id)
            .expect("city to read income from exists");
        let centre = self.map.tile_at(city.location);
        let mut food = centre.yields_food() as u32;
        let mut resources = centre.yields_resources() as u32;
        for location in city.worked_tiles() {
            let tile = self.map.tile_at(*location);
            food += tile.yields_food() as u32;
            resources += tile.yields_resources() as u32;
        }
        (food, resources)
    }

    /// Total beakers produced by all of a player's cities this turn.
    pub fn research_income(&self, owner: PlayerId) -> u32 {
        self.cities
            .iter()
            .filter(|city| city.owner() == owner)
            .map(|city| city.research())
            .sum()
    }

    /// Whether a player may begin researching `advancement`: it must not be
    /// already discovered and all of its prerequisites must have been.
    pub fn can_research(&self, owner: PlayerId, advancement: Advancement) -> bool {
        let player = &self.players[owner.index()];
        if player.has_advancement(advancement) {
            return false;
        }
        advancement
            .prerequisites()
            .iter()
            .all(|prereq| player.has_advancement(*prereq))
    }

    pub(crate) fn begin_research(&mut self, owner: PlayerId) {
        self.players[owner.index()].begin_research();
    }

    pub fn advancement_in_progress(&self, owner: PlayerId) -> Option<Advancement> {
        self.players[owner.index()].advancement_in_progress()
    }

    pub fn research_progress(&self, owner: PlayerId) -> u32 {
        self.players[owner.index()].research_progress()
    }

    pub(crate) fn set_research_target(&mut self, owner: PlayerId, advancement: Advancement) {
        self.players[owner.index()].set_research_target(advancement);
    }

    pub(crate) fn advance_research(&mut self, owner: PlayerId) -> Option<Advancement> {
        let beakers = self.research_income(owner);
        self.players[owner.index()].advance_research(beakers)
    }

    /// The Chebyshev radius-2 footprint around a city (the 21 fog-reveal tiles),
    /// wrapped east/west and clamped north/south.
    pub fn city_footprint(&self, location: Location) -> Vec<Location> {
        let mut result = Vec::new();
        let x0 = location.x as isize;
        let y0 = location.y as isize;
        for y in (y0 - 2)..=(y0 + 2) {
            if y < 0 || y >= self.map.height as isize {
                continue;
            }
            for x in (x0 - 2)..=(x0 + 2) {
                if (x - x0).abs() == 2 && (y - y0).abs() == 2 {
                    continue;
                }
                let px = x.rem_euclid(self.map.width as isize) as u16;
                result.push(Location::new(px, y as u16));
            }
        }
        result
    }

    /// Assign worked tiles so each citizen harvests a ring tile within the
    /// footprint (the city centre is additionally always worked), preferring
    /// the most resource-productive available tiles.
    pub fn auto_assign_work(&mut self, city_id: CityId) {
        let city = self
            .cities
            .iter()
            .find(|city| city.id() == city_id)
            .expect("city to assign work for exists");
        let city_location = city.location;
        let city_population = city.population() as usize;
        let currently_worked_tiles_locations: Vec<Location> = city.worked_tiles().to_vec();

        if currently_worked_tiles_locations.len() >= city_population {
            return;
        }

        let mut candidates: Vec<Location> = self
            .city_footprint(city_location)
            .into_iter()
            .filter(|candidate| {
                *candidate != city_location && !currently_worked_tiles_locations.contains(candidate)
            })
            .collect();
        candidates.sort_by_key(|candidate| {
            let tile = self.map.tile_at(*candidate);
            std::cmp::Reverse((tile.yields_resources(), tile.yields_food()))
        });

        let to_add = city_population - currently_worked_tiles_locations.len();
        for candidate in candidates.into_iter().take(to_add) {
            let city = self
                .cities
                .iter_mut()
                .find(|city| city.id() == city_id)
                .expect("city to assign work for exists");
            city.add_worked_tile(candidate);
        }
    }

    /// Tick a city for one turn, delegating the economy to the model.
    pub(crate) fn process_city(&mut self, city_id: CityId) -> CityProcess {
        let (food_income, resource_income) = self.city_income(city_id);
        let tick = self
            .cities
            .iter_mut()
            .find(|city| city.id() == city_id)
            .expect("city to process exists")
            .tick(food_income, resource_income);
        CityProcess {
            produced: tick.produced,
            grew: tick.grew,
            completed: tick.completed,
            starving: tick.starving,
        }
    }
}

/// Outcome of processing a single city for one turn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CityProcess {
    pub produced: u32,
    pub grew: bool,
    pub completed: Option<ProductionTarget>,
    pub starving: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::cartography::Tile;
    use crate::model::cities::CityImprovement;
    use crate::model::civilizations::Civilization;
    use crate::model::geography::Geography;

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

    #[test]
    fn players_start_alone_and_unmet() {
        let game = Game::new(
            3,
            2,
            Player::new(Civilization::English),
            vec![Player::new(Civilization::Zulu)],
        );
        assert!(!game.at_war(PlayerId::new(0), PlayerId::new(1)));
        assert!(!game.at_peace(PlayerId::new(0), PlayerId::new(1)));
        assert!(!game.at_war(PlayerId::new(0), PlayerId::new(0)));
        assert!(game.players[0].at_war_with().is_empty());
        assert!(game.players[0].at_peace_with().is_empty());
    }

    #[test]
    fn declaring_war_registers_on_both_players() {
        let mut game = Game::new(
            3,
            2,
            Player::new(Civilization::English),
            vec![
                Player::new(Civilization::Zulu),
                Player::new(Civilization::Roman),
            ],
        );
        game.declare_war(PlayerId::new(0), PlayerId::new(2));
        assert!(game.at_war(PlayerId::new(0), PlayerId::new(2)));
        assert!(game.at_war(PlayerId::new(2), PlayerId::new(0)));
        assert!(game.players[0].at_war_with().contains(&PlayerId::new(2)));
        assert!(game.players[2].at_war_with().contains(&PlayerId::new(0)));
        assert!(!game.at_war(PlayerId::new(0), PlayerId::new(1)));
    }

    #[test]
    fn declaring_war_after_meeting_in_peace_updates_both_lists() {
        let mut game = Game::new(
            3,
            2,
            Player::new(Civilization::English),
            vec![Player::new(Civilization::Zulu)],
        );
        game.make_peace(PlayerId::new(0), PlayerId::new(1));
        assert!(game.at_peace(PlayerId::new(0), PlayerId::new(1)));
        assert!(game.players[0].at_peace_with().contains(&PlayerId::new(1)));
        assert!(game.players[1].at_peace_with().contains(&PlayerId::new(0)));
        game.declare_war(PlayerId::new(0), PlayerId::new(1));
        assert!(game.at_war(PlayerId::new(0), PlayerId::new(1)));
        assert!(!game.at_peace(PlayerId::new(0), PlayerId::new(1)));
        assert!(game.players[0].at_peace_with().is_empty());
        assert!(game.players[1].at_peace_with().is_empty());
    }

    #[test]
    fn making_peace_after_a_war_ends_it() {
        let mut game = Game::new(
            3,
            2,
            Player::new(Civilization::English),
            vec![Player::new(Civilization::Zulu)],
        );
        game.declare_war(PlayerId::new(0), PlayerId::new(1));
        game.make_peace(PlayerId::new(0), PlayerId::new(1));
        assert!(!game.at_war(PlayerId::new(0), PlayerId::new(1)));
        assert!(game.at_peace(PlayerId::new(0), PlayerId::new(1)));
    }

    #[test]
    fn met_reflects_war_or_peace() {
        let mut game = Game::new(
            3,
            2,
            Player::new(Civilization::English),
            vec![Player::new(Civilization::Zulu)],
        );
        assert!(!game.met(PlayerId::new(0), PlayerId::new(1)));
        game.make_peace(PlayerId::new(0), PlayerId::new(1));
        assert!(game.met(PlayerId::new(0), PlayerId::new(1)));
        assert!(game.met(PlayerId::new(1), PlayerId::new(0)));
        game.declare_war(PlayerId::new(0), PlayerId::new(1));
        assert!(game.met(PlayerId::new(0), PlayerId::new(1)));
        assert!(!game.met(PlayerId::new(0), PlayerId::new(0)));
    }

    #[test]
    fn revealing_around_a_city_marks_its_footprint_for_the_owner() {
        let mut game = Game::new(
            5,
            5,
            Player::new(Civilization::English),
            vec![Player::new(Civilization::Zulu)],
        );
        game.reveal_tiles_surrounding_city_at(PlayerId::new(0), Location::new(2, 2));
        for y in 0..5 {
            for x in 0..5 {
                let corner = (x == 0 || x == 4) && (y == 0 || y == 4);
                assert_eq!(
                    game.players[0].explored_at(x, y),
                    !corner,
                    "tile ({x}, {y})"
                );
            }
        }
        assert!(!game.players[1].explored_at(2, 2));
    }

    #[test]
    fn disbanding_units_sorted_by_home_city_removes_only_those() {
        let mut game = Game::new(3, 2, Player::new(Civilization::English), Vec::new());
        let first = game.spawn_unit(UnitClass::Legion, Location::new(0, 0), player(), home());
        let second = game.spawn_unit(UnitClass::Legion, Location::new(1, 0), player(), home());
        let other = game.spawn_unit(
            UnitClass::Diplomat,
            Location::new(2, 0),
            player(),
            CityId::new(1),
        );
        assert_eq!(game.disband_units_homed_to(home()), 2);
        assert_eq!(game.units.len(), 1);
        assert!(game.units.iter().any(|unit| unit.id() == other));
        assert!(!game.units.iter().any(|unit| unit.id() == first));
        assert!(!game.units.iter().any(|unit| unit.id() == second));
    }

    #[test]
    fn disbanding_with_no_matching_home_disbands_nothing() {
        let mut game = Game::new(3, 2, Player::new(Civilization::English), Vec::new());
        game.spawn_unit(
            UnitClass::Legion,
            Location::new(0, 0),
            player(),
            CityId::new(1),
        );
        assert_eq!(game.disband_units_homed_to(home()), 0);
        assert_eq!(game.units.len(), 1);
    }

    #[test]
    fn owned_units_returns_only_that_owners_units() {
        let mut game = Game::new(
            3,
            2,
            Player::new(Civilization::English),
            vec![Player::new(Civilization::Zulu)],
        );
        let mine_first = game.spawn_unit(UnitClass::Settler, Location::new(0, 0), player(), home());
        let mine_second = game.spawn_unit(UnitClass::Legion, Location::new(1, 0), player(), home());
        let theirs = game.spawn_unit(
            UnitClass::Diplomat,
            Location::new(2, 0),
            PlayerId::new(1),
            CityId::new(0),
        );
        assert_eq!(game.owned_units(player()), vec![mine_first, mine_second]);
        assert_eq!(game.owned_units(PlayerId::new(1)), vec![theirs]);
    }

    #[test]
    fn city_income_sums_city_centre_and_worked_tiles() {
        let mut game = Game::new(5, 5, Player::new(Civilization::English), Vec::new());
        let london = game.add_city(player(), "London", Location::new(2, 2));
        let grassland = Location::new(1, 2);
        *game.map.tile_at_mut(grassland) = Tile::new(Geography::Grassland);
        let mut plains = Tile::new(Geography::Plains);
        plains.irrigate().unwrap();
        *game.map.tile_at_mut(Location::new(3, 2)) = plains;
        game.cities[0].add_worked_tile(grassland);
        game.cities[0].add_worked_tile(Location::new(3, 2));
        // centre ocean 2/0 + grassland 2/0 + irrigated plains 3/1
        assert_eq!(game.city_income(london), (7, 1));
    }

    #[test]
    fn research_income_sums_the_players_city_research() {
        let mut game = Game::new(3, 2, Player::new(Civilization::English), Vec::new());
        game.add_city(player(), "London", Location::new(1, 0));
        game.add_city(player(), "York", Location::new(1, 1));
        assert_eq!(game.research_income(player()), 2);
        for _ in 0..3 {
            game.cities[0].grow();
        }
        assert_eq!(game.research_income(player()), 5);
        game.cities[0].add_improvement(CityImprovement::Library);
        assert_eq!(game.research_income(player()), 7);
    }

    #[test]
    fn can_research_requires_prerequisites_and_newness() {
        let mut game = Game::new(3, 1, Player::new(Civilization::English), Vec::new());
        game.add_city(player(), "London", Location::new(1, 0));
        for _ in 0..8 {
            game.cities[0].grow();
        }
        game.cities[0].add_improvement(CityImprovement::Library);
        game.cities[0].add_improvement(CityImprovement::University);
        assert_eq!(game.research_income(player()), 20);

        assert!(game.can_research(player(), Advancement::Wheel));
        assert!(!game.can_research(player(), Advancement::Currency));
        assert!(!game.can_research(player(), Advancement::Astronomy));

        game.set_research_target(player(), Advancement::Wheel);
        assert_eq!(game.advance_research(player()), None);
        assert_eq!(game.advance_research(player()), Some(Advancement::Wheel));
        assert!(!game.can_research(player(), Advancement::Wheel));

        game.set_research_target(player(), Advancement::BronzeWorking);
        assert_eq!(game.advance_research(player()), None);
        assert_eq!(
            game.advance_research(player()),
            Some(Advancement::BronzeWorking)
        );
        assert!(game.can_research(player(), Advancement::Currency));
    }

    #[test]
    fn research_state_is_exposed_through_the_game() {
        let mut game = Game::new(3, 2, Player::new(Civilization::English), Vec::new());
        assert_eq!(game.advancement_in_progress(player()), None);
        game.begin_research(player());
        assert_eq!(
            game.advancement_in_progress(player()),
            Some(Advancement::Construction)
        );
        assert_eq!(game.research_progress(player()), 0);
        game.set_research_target(player(), Advancement::Wheel);
        assert_eq!(
            game.advancement_in_progress(player()),
            Some(Advancement::Wheel)
        );
    }

    #[test]
    fn research_progress_accumulates_and_resets_on_discovery() {
        let mut game = Game::new(3, 2, Player::new(Civilization::English), Vec::new());
        game.add_city(player(), "London", Location::new(1, 0));
        for _ in 0..3 {
            game.cities[0].grow();
        }
        game.cities[0].add_improvement(CityImprovement::Library);
        assert_eq!(game.research_income(player()), 6);

        game.set_research_target(player(), Advancement::Wheel);
        game.advance_research(player());
        assert_eq!(game.research_progress(player()), 6);
        for _ in 0..6 {
            game.advance_research(player());
        }
        assert_eq!(game.research_progress(player()), 0);
        assert_eq!(game.advancement_in_progress(player()), None);
        assert!(game.players[0].has_advancement(Advancement::Wheel));
    }

    #[test]
    fn city_footprint_is_the_21_tile_ring_diamond() {
        let game = Game::new(7, 7, Player::new(Civilization::English), Vec::new());
        let footprint = game.city_footprint(Location::new(3, 3));
        assert_eq!(footprint.len(), 21);
        assert!(footprint.contains(&Location::new(3, 3)));
        assert!(footprint.contains(&Location::new(3, 1)));
        assert!(footprint.contains(&Location::new(5, 3)));
        assert!(footprint.contains(&Location::new(1, 2)));
        assert!(!footprint.contains(&Location::new(1, 1)));
        assert!(!footprint.contains(&Location::new(5, 5)));
    }

    #[test]
    fn city_footprint_wraps_around_east_and_west() {
        let game = Game::new(5, 5, Player::new(Civilization::English), Vec::new());
        let footprint = game.city_footprint(Location::new(0, 2));
        assert!(footprint.contains(&Location::new(4, 2)));
        assert!(footprint.contains(&Location::new(1, 2)));
        assert_eq!(footprint.len(), 21);
    }

    #[test]
    fn city_footprint_clamps_north_and_south() {
        let game = Game::new(5, 5, Player::new(Civilization::English), Vec::new());
        let footprint = game.city_footprint(Location::new(2, 0));
        assert!(footprint.contains(&Location::new(2, 0)));
        assert!(footprint.contains(&Location::new(2, 2)));
        assert!(!footprint.iter().any(|location| location.y >= 3));
        assert_eq!(footprint.len(), 13);
    }

    #[test]
    fn auto_assign_work_chooses_the_most_resource_productive_tiles() {
        let mut game = Game::new(5, 5, Player::new(Civilization::English), Vec::new());
        let london = game.add_city(player(), "London", Location::new(2, 2));
        game.cities[0].grow();
        *game.map.tile_at_mut(Location::new(3, 2)) = Tile::new(Geography::Desert);
        *game.map.tile_at_mut(Location::new(1, 2)) = Tile::new(Geography::Desert);
        game.auto_assign_work(london);
        let worked = game.cities[0].worked_tiles().to_vec();
        assert_eq!(worked.len(), 2);
        assert!(worked.contains(&Location::new(3, 2)));
        assert!(worked.contains(&Location::new(1, 2)));
    }

    #[test]
    fn auto_assign_work_leaves_a_fully_worked_city_alone() {
        let mut game = Game::new(5, 5, Player::new(Civilization::English), Vec::new());
        let london = game.add_city(player(), "London", Location::new(2, 2));
        game.cities[0].add_worked_tile(Location::new(3, 2));
        game.auto_assign_work(london);
        assert_eq!(game.cities[0].worked_tiles().len(), 1);
    }

    #[test]
    fn processing_a_city_grows_it_and_reports_the_outcome() {
        let mut game = Game::new(3, 2, Player::new(Civilization::English), Vec::new());
        let mut centre = Tile::new(Geography::Grassland);
        centre.irrigate().unwrap();
        *game.map.tile_at_mut(Location::new(1, 0)) = centre;
        let london = game.add_city(player(), "London", Location::new(1, 0));
        let first = game.process_city(london);
        assert!(!first.grew);
        assert!(!first.starving);
        let second = game.process_city(london);
        assert!(second.grew);
        assert_eq!(game.cities[0].population(), 2);
    }

    #[test]
    fn processing_a_starving_city_reports_starvation() {
        let mut game = Game::new(3, 2, Player::new(Civilization::English), Vec::new());
        *game.map.tile_at_mut(Location::new(1, 0)) = Tile::new(Geography::Mountain);
        let london = game.add_city(player(), "London", Location::new(1, 0));
        let outcome = game.process_city(london);
        assert!(outcome.starving);
        assert_eq!(outcome.produced, 1);
        assert!(!outcome.grew);
        assert_eq!(game.cities[0].food(), 0);
    }

    #[test]
    fn processing_a_city_completes_production_when_resources_suffice() {
        let mut game = Game::new(3, 2, Player::new(Civilization::English), Vec::new());
        let mut centre = Tile::new(Geography::Mountain);
        centre.mine().unwrap();
        *game.map.tile_at_mut(Location::new(1, 0)) = centre;
        let london = game.add_city(player(), "London", Location::new(1, 0));
        game.cities[0].set_production(ProductionTarget::Unit(UnitClass::Militia));
        for _ in 0..4 {
            let outcome = game.process_city(london);
            assert_eq!(outcome.completed, None);
        }
        let fifth = game.process_city(london);
        assert_eq!(
            fifth.completed,
            Some(ProductionTarget::Unit(UnitClass::Militia))
        );
        assert_eq!(game.cities[0].production_target(), None);
        assert_eq!(game.cities[0].resource_stored(), 0);
    }
}
