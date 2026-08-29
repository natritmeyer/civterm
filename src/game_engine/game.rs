use crate::game_engine::player::Player;
use crate::model::cartography::Map;

pub struct Game {
    pub map: Map,
    pub players: Vec<Player>,
}

impl Game {
    pub fn new(width: usize, height: usize, first: Player, rest: Vec<Player>) -> Self {
        let mut players = Vec::with_capacity(rest.len() + 1);
        players.push(first);
        players.extend(rest);
        Game {
            map: Map::new(width, height),
            players,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::civilizations::Civilization;

    #[test]
    fn game_created_with_the_given_players() {
        let game = Game::new(
            3,
            2,
            Player::new(Civilization::English),
            vec![Player::new(Civilization::Zulu)],
        );
        assert_eq!(
            game.players,
            vec![
                Player::new(Civilization::English),
                Player::new(Civilization::Zulu)
            ]
        );
    }

    #[test]
    fn game_can_have_a_single_player() {
        let game = Game::new(3, 2, Player::new(Civilization::Roman), Vec::new());
        assert_eq!(game.players, vec![Player::new(Civilization::Roman)]);
    }

    #[test]
    fn game_map_has_requested_dimensions() {
        let game = Game::new(3, 2, Player::new(Civilization::English), Vec::new());
        assert_eq!(game.map.width, 3);
        assert_eq!(game.map.height, 2);
    }
}
