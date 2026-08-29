use crate::model::advancements::Advancement;
use crate::model::cities::City;
use crate::model::civilizations::Civilization;

#[derive(Clone, Debug, PartialEq)]
pub struct Player {
    pub civilization: Civilization,
    pub cities: Vec<City>,
    advancement_in_progress: Option<Advancement>,
    advances_made: Vec<Advancement>,
}

impl Player {
    pub fn new(civilization: Civilization) -> Self {
        Player {
            civilization,
            cities: Vec::new(),
            advancement_in_progress: None,
            advances_made: Vec::new(),
        }
    }

    pub fn advancement_in_progress(&self) -> Option<Advancement> {
        self.advancement_in_progress
    }

    pub fn advances_made(&self) -> &[Advancement] {
        &self.advances_made
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
    fn player_starts_with_no_cities() {
        let player = Player::new(Civilization::English);
        assert!(player.cities.is_empty());
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
