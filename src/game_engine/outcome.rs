use crate::game_engine::engine::Engine;
use crate::game_engine::player::Player;

/// The two ways a match can end, both derived from the permanent elimination
/// of civilizations: the human's own fall is a defeat, and the loss of every
/// rival is a victory. No engine state records it, so a save needs no new
/// field and a reload re-derives the outcome from the loaded game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameOutcome {
    Victory,
    Defeat,
}

impl Engine {
    /// How the current match has ended, or `None` while both the human and at
    /// least one rival are still in play. Defeat is checked first: the human
    /// losing their last city — or their last unit with no city to their name
    /// — always reads as defeat, even if the rivals are already gone in the
    /// same round.
    pub fn game_outcome(&self) -> Option<GameOutcome> {
        let players = &self.game.players;
        if players.first().is_some_and(Player::eliminated) {
            return Some(GameOutcome::Defeat);
        }
        if players.len() > 1 && players[1..].iter().all(Player::eliminated) {
            return Some(GameOutcome::Victory);
        }
        None
    }
}
