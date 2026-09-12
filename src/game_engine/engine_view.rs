use crate::game_engine::calendar::calendar_year;
use crate::game_engine::{CityIncome, Engine, GameView};
use crate::model::advancements::Advancement;
use crate::model::cartography::{Location, Tile};
use crate::model::cities::{City, CityId, ProductionTarget};
use crate::model::civilizations::{Civilization, PlayerId};
use crate::model::units::Unit;

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

    fn player_cities(&self) -> Vec<&City> {
        self.game
            .cities
            .iter()
            .filter(|city| city.owner() == self.current_player_index)
            .collect()
    }

    fn city(&self, id: CityId) -> Option<&City> {
        self.game.cities.iter().find(|city| city.id() == id)
    }

    fn current_player_id(&self) -> PlayerId {
        self.current_player_index
    }

    fn city_income(&self, id: CityId) -> CityIncome {
        if self.game.cities.iter().any(|city| city.id() == id) {
            self.game.city_breakdown(id)
        } else {
            CityIncome {
                food: 0,
                resources: 0,
                trade: 0,
                gold: 0,
                research: 0,
                special_resources: Vec::new(),
            }
        }
    }

    fn home_units(&self, city: CityId) -> Vec<&Unit> {
        self.game
            .units
            .iter()
            .filter(|unit| unit.home_city() == city)
            .collect()
    }

    fn player_units(&self) -> Vec<&Unit> {
        self.game
            .units
            .iter()
            .filter(|unit| unit.owner() == self.current_player_index)
            .collect()
    }

    fn explored(&self, x: usize, y: usize) -> bool {
        self.game.players[self.current_player_index.index()].explored_at(x, y)
    }

    fn current_player(&self) -> Civilization {
        self.game.players[self.current_player_index.index()].civilization
    }

    fn civilization_of(&self, player: PlayerId) -> Civilization {
        self.game.players[player.index()].civilization
    }

    fn turn(&self) -> u32 {
        self.turn
    }

    fn year(&self) -> i32 {
        calendar_year(self.turn)
    }

    fn gold(&self) -> u32 {
        self.game.players[self.current_player_index.index()].gold()
    }

    fn advancement_in_progress(&self) -> Option<Advancement> {
        self.game.advancement_in_progress(self.current_player_index)
    }

    fn research_progress(&self) -> u32 {
        self.game.research_progress(self.current_player_index)
    }

    fn research_cost(&self) -> Option<u32> {
        self.game
            .advancement_in_progress(self.current_player_index)
            .map(|advancement| advancement.cost())
    }

    fn research_income(&self) -> u32 {
        self.game.research_income(self.current_player_index)
    }

    fn production_choices(&self, city: CityId) -> Vec<ProductionTarget> {
        let player = &self.game.players[self.current_player_index.index()];
        let owned_improvements = self
            .game
            .cities
            .iter()
            .find(|owned| owned.id() == city && owned.owner() == self.current_player_index)
            .map(|owned| owned.improvements().to_vec())
            .unwrap_or_default();
        player
            .available_unit_classes()
            .into_iter()
            .map(ProductionTarget::Unit)
            .chain(
                player
                    .available_improvements()
                    .into_iter()
                    .filter(|improvement| !owned_improvements.contains(improvement))
                    .map(ProductionTarget::Improvement),
            )
            .collect()
    }
}
