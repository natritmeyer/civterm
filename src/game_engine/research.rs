use super::*;

use crate::game_engine::Event;
use crate::model::advancements::Advancement;
use crate::model::civilizations::PlayerId;

impl Engine {
    /// Aggregate one turn of civilisations' city research into advancement
    /// progress at the player level.
    pub(super) fn process_research(&mut self, owner: PlayerId) {
        if let Some(advancement) = self.game.advance_research(owner) {
            self.events.push(Event::new(format!(
                "{:?} discover {:?}",
                self.game.players[owner.index()].civilization,
                advancement
            )));
        }
    }
    pub(super) fn set_research_target(&mut self, advancement: Advancement) {
        let owner = self.current_player_index;
        if !self.game.can_research(owner, advancement) {
            let player = &self.game.players[owner.index()];
            let reason = if player.has_advancement(advancement) {
                "already discovered"
            } else {
                "prerequisites not met"
            };
            self.events.push(Event::new(format!(
                "Cannot research {:?}: {}",
                advancement, reason
            )));
            return;
        }
        self.game.set_research_target(owner, advancement);
        self.events.push(Event::new(format!(
            "{:?} begin researching {:?}",
            self.game.players[owner.index()].civilization,
            advancement
        )));
    }
}
