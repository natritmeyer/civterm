use crate::model::advancements::Advancement;
use crate::model::cartography::Tile;
use crate::model::cities::{City, CityId};
use crate::model::civilizations::{Civilization, PlayerId};
use crate::model::geography::SpecialResource;
use crate::model::units::Unit;

/// What a city harvests in a single turn from its centre and worked tiles.
pub struct CityIncome {
    pub food: u32,
    pub resources: u32,
    pub trade: u32,
    pub gold: u32,
    pub research: u32,
    /// The distinct special resources being worked by this city.
    pub special_resources: Vec<SpecialResource>,
}

pub trait GameView {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn tile(&self, x: usize, y: usize) -> &Tile;
    fn units_at(&self, x: usize, y: usize) -> Vec<&Unit>;
    fn city_at(&self, x: usize, y: usize) -> Option<&City>;
    /// All units owned by the current player.
    fn player_units(&self) -> Vec<&Unit>;
    /// All cities owned by the current player.
    fn player_cities(&self) -> Vec<&City>;
    /// The city with the given id, anywhere on the map.
    fn city(&self, id: CityId) -> Option<&City>;
    /// The player id of the civ whose turn it is.
    fn current_player_id(&self) -> PlayerId;
    /// This turn's harvest for the given city, or all zeros if there is no
    /// such city.
    fn city_income(&self, id: CityId) -> CityIncome;
    /// The units whose home city is `city`.
    fn home_units(&self, city: CityId) -> Vec<&Unit>;
    fn explored(&self, x: usize, y: usize) -> bool;
    fn current_player(&self) -> Civilization;
    /// The civilization governing the given player.
    fn civilization_of(&self, player: PlayerId) -> Civilization;
    fn turn(&self) -> u32;
    /// The calendar year corresponding to the current turn.
    fn year(&self) -> i32;
    /// The current player's treasury.
    fn gold(&self) -> u32;
    /// The advancement the current player is researching, if any.
    fn advancement_in_progress(&self) -> Option<Advancement>;
    /// Beakers accumulated toward the current research target.
    fn research_progress(&self) -> u32;
    /// Beakers required to complete the current research target, if one is set.
    fn research_cost(&self) -> Option<u32>;
    /// Research income per turn for the current player.
    fn research_income(&self) -> u32;
}
