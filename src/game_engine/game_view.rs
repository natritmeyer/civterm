use crate::model::advancements::Advancement;
use crate::model::cartography::Tile;
use crate::model::cities::City;
use crate::model::civilizations::{Civilization, PlayerId};
use crate::model::units::Unit;

pub trait GameView {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn tile(&self, x: usize, y: usize) -> &Tile;
    fn units_at(&self, x: usize, y: usize) -> Vec<&Unit>;
    fn city_at(&self, x: usize, y: usize) -> Option<&City>;
    /// All units owned by the current player.
    fn player_units(&self) -> Vec<&Unit>;
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
