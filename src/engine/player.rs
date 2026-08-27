use crate::model::city::City;
use crate::model::civilization::Civilization;
use crate::model::unit::Unit;

#[derive(Clone, Debug, PartialEq)]
pub struct Player {
    pub civilization: Civilization,
    pub cities: Vec<City>,
    pub units: Vec<Unit>,
}

impl Player {
    pub fn new(civilization: Civilization) -> Self {
        Player {
            civilization,
            cities: Vec::new(),
            units: Vec::new(),
        }
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
    fn player_starts_with_no_cities_or_units() {
        let player = Player::new(Civilization::English);
        assert!(player.cities.is_empty());
        assert!(player.units.is_empty());
    }
}
