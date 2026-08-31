use crate::game_engine::Exploration;
use crate::model::advancements::Advancement;
use crate::model::cartography::Location;
use crate::model::civilizations::{Civilization, PlayerId};

#[derive(Clone, Debug, PartialEq)]
pub struct Player {
    pub civilization: Civilization,
    advancement_in_progress: Option<Advancement>,
    advances_made: Vec<Advancement>,
    explored: Exploration,
    pub(super) at_war_with: Vec<PlayerId>,
    pub(super) at_peace_with: Vec<PlayerId>,
}

impl Player {
    pub fn new(civilization: Civilization) -> Self {
        Player {
            civilization,
            advancement_in_progress: None,
            advances_made: Vec::new(),
            explored: Exploration::empty(),
            at_war_with: Vec::new(),
            at_peace_with: Vec::new(),
        }
    }

    pub fn at_war_with(&self) -> &[PlayerId] {
        &self.at_war_with
    }

    pub fn at_peace_with(&self) -> &[PlayerId] {
        &self.at_peace_with
    }

    pub(super) fn enter_war_with(&mut self, other: PlayerId) {
        if !self.at_war_with.contains(&other) {
            self.at_war_with.push(other);
        }
        self.at_peace_with.retain(|player| *player != other);
    }

    pub(super) fn enter_peace_with(&mut self, other: PlayerId) {
        if !self.at_peace_with.contains(&other) {
            self.at_peace_with.push(other);
        }
        self.at_war_with.retain(|player| *player != other);
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
        self.explored.discovered(x, y)
    }

    pub fn reveal_tiles_at(&mut self, origin: Location, radius: u8) {
        self.explored.reveal_tiles_at(origin, radius);
    }

    pub fn reveal_tiles_surrounding_city_at(&mut self, origin: Location) {
        self.explored.reveal_tiles_surrounding_city_at(origin);
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

    #[test]
    fn a_player_has_met_nobody_at_the_start() {
        let player = Player::new(Civilization::English);
        assert!(player.at_war_with().is_empty());
        assert!(player.at_peace_with().is_empty());
    }

    #[test]
    fn entering_war_removes_a_previous_peace() {
        let mut player = Player::new(Civilization::English);
        player.enter_peace_with(PlayerId::new(1));
        player.enter_war_with(PlayerId::new(1));
        assert!(player.at_war_with().contains(&PlayerId::new(1)));
        assert!(!player.at_peace_with().contains(&PlayerId::new(1)));
    }

    #[test]
    fn entering_peace_removes_a_previous_war() {
        let mut player = Player::new(Civilization::English);
        player.enter_war_with(PlayerId::new(1));
        player.enter_peace_with(PlayerId::new(1));
        assert!(!player.at_war_with().contains(&PlayerId::new(1)));
        assert!(player.at_peace_with().contains(&PlayerId::new(1)));
    }
}
