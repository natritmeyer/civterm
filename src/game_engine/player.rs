use crate::game_engine::Exploration;
use crate::model::advancements::Advancement;
use crate::model::cartography::Location;
use crate::model::civilizations::Civilization;

#[derive(Clone, Debug, PartialEq)]
pub struct Player {
    pub civilization: Civilization,
    advancement_in_progress: Option<Advancement>,
    advances_made: Vec<Advancement>,
    explored: Exploration,
}

impl Player {
    pub fn new(civilization: Civilization) -> Self {
        Player {
            civilization,
            advancement_in_progress: None,
            advances_made: Vec::new(),
            explored: Exploration::empty(),
        }
    }

    pub fn advancement_in_progress(&self) -> Option<Advancement> {
        self.advancement_in_progress
    }

    pub fn advances_made(&self) -> &[Advancement] {
        &self.advances_made
    }

    pub(super) fn seed_exploration(&mut self, width: usize, height: usize) {
        self.explored = Exploration::new(width, height);
    }

    pub fn explored_at(&self, x: usize, y: usize) -> bool {
        self.explored.marks(x, y)
    }

    pub fn reveal(&mut self, origin: Location, radius: u8) {
        self.explored.reveal(origin, radius);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_is_created_with_given_civilization() {
        let player = Player::new(Civilization::English);
        assert_eq!(player.civilization, Civilization::English);
    }

    #[test]
    fn player_starts_with_no_advancement_in_progress() {
        let player = Player::new(Civilization::English);
        assert_eq!(player.advancement_in_progress(), None);
    }

    #[test]
    fn player_starts_with_no_advances_made() {
        let player = Player::new(Civilization::English);
        assert!(player.advances_made().is_empty());
    }
}
