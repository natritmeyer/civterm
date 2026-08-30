use crate::model::cartography::Tile;
use crate::model::cities::City;
use crate::model::civilizations::Civilization;
use crate::model::units::Unit;

pub trait GameView {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn tile(&self, x: usize, y: usize) -> &Tile;
    fn units_at(&self, x: usize, y: usize) -> Vec<&Unit>;
    fn city_at(&self, x: usize, y: usize) -> Option<&City>;
    fn explored(&self, x: usize, y: usize) -> bool;
    fn current_player(&self) -> Civilization;
    fn turn(&self) -> u32;
}
