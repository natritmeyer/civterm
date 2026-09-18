use super::*;

use crate::game_engine::{Event, GameView};
use crate::model::civilizations::PlayerId;
use crate::model::units::UnitOrder;

impl Engine {
    /// Resolve a whole round: every non-human player takes their turn in
    /// order, then control returns to the human with the turn number advanced
    /// by one. Each rival's turn begins with the same restore/process
    /// preamble as any other, followed by the rival AI's decisions.
    pub(super) fn end_turn(&mut self) {
        let player_count = self.game.players.len();
        let mut rivals_acted = 0;
        loop {
            self.advance_to_next_player();
            self.begin_turn();
            if self.current_player_index == PlayerId::new(0) {
                self.turn += 1;
                break;
            }
            self.events.push(Event::new(format!(
                "{:?} begins turn {}",
                self.current_player(),
                self.turn
            )));
            self.run_rival_turn();
            rivals_acted += 1;
            // Only the human's turn ends a round. If the human has been
            // eliminated, advancement skips straight back to the rivals, so a
            // full lap without reaching the human must stop the loop rather
            // than spin forever.
            if rivals_acted >= player_count {
                break;
            }
        }
        self.events.push(Event::new(format!(
            "{:?} begins turn {}",
            self.current_player(),
            self.turn
        )));
    }
    pub(super) fn advance_to_next_player(&mut self) {
        let count = self.game.players.len();
        let mut next = (self.current_player_index.index() + 1) % count;
        let mut considered = 0;
        while considered < count && self.game.players[next].eliminated() {
            next = (next + 1) % count;
            considered += 1;
        }
        self.current_player_index = PlayerId::new(next);
    }
    pub(super) fn begin_turn(&mut self) {
        for unit in self
            .game
            .units
            .iter_mut()
            .filter(|unit| unit.owner() == self.current_player_index)
        {
            if let UnitOrder::Improving(improvement) = unit.order() {
                // A settler mid-build gets no moves back; it works the tile
                // instead. When the last turn lands, the improvement is
                // applied and the settler's turn is spent finishing it.
                unit.advance_work();
                if unit.work_progress() >= improvement.work_turns() {
                    let location = unit.location;
                    let done = unit.id();
                    let finished = self
                        .game
                        .map
                        .tile_at_mut(location)
                        .apply_improvement(improvement)
                        .is_ok();
                    unit.cancel_order();
                    unit.spend_turn();
                    self.events.push(Event::new(if finished {
                        format!("Unit {} finishes {:?}", done.index(), improvement)
                    } else {
                        format!("Cannot build {:?} here", improvement)
                    }));
                }
            } else {
                unit.restore_moves();
            }
        }
        self.process_cities(self.current_player_index);
        self.process_research(self.current_player_index);
    }
}
