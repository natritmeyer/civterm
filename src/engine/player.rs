use crate::model::civilization::Civilization;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Player {
    pub civilization: Civilization,
}

impl Player {
    pub fn new(civilization: Civilization) -> Self {
        Player { civilization }
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
}
