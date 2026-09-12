use super::*;

use crate::game_engine::Event;
use crate::model::cartography::Location;
use crate::model::civilizations::PlayerId;

impl Engine {
    pub(super) fn declare_war(&mut self, opponent: PlayerId) {
        if opponent == self.current_player_index {
            self.events
                .push(Event::new("Cannot declare war on yourself"));
            return;
        }
        if opponent.index() >= self.game.players.len() {
            self.events.push(Event::new("No such player"));
            return;
        }
        if self.game.at_war(self.current_player_index, opponent) {
            self.events.push(Event::new("Already at war"));
            return;
        }
        self.game.declare_war(self.current_player_index, opponent);
        self.events.push(Event::new(format!(
            "{:?} declares war on {:?}",
            self.game.players[self.current_player_index.index()].civilization,
            self.game.players[opponent.index()].civilization
        )));
    }
    pub(super) fn make_peace(&mut self, opponent: PlayerId) {
        if opponent == self.current_player_index {
            self.events
                .push(Event::new("Cannot make peace with yourself"));
            return;
        }
        if opponent.index() >= self.game.players.len() {
            self.events.push(Event::new("No such player"));
            return;
        }
        if self.game.at_peace(self.current_player_index, opponent) {
            self.events.push(Event::new("Already at peace"));
            return;
        }
        self.game.make_peace(self.current_player_index, opponent);
        self.events.push(Event::new(format!(
            "{:?} makes peace with {:?}",
            self.game.players[self.current_player_index.index()].civilization,
            self.game.players[opponent.index()].civilization
        )));
    }
    pub(super) fn meet_contacts_within(&mut self, location: Location, owner: PlayerId) {
        let width = self.game.map.width;
        let height = self.game.map.height;
        let mut contacts: Vec<PlayerId> = Vec::new();
        for dy in -1..=1 {
            let y = location.y as i32 + dy;
            if y < 0 || y >= height as i32 {
                continue;
            }
            for dx in -1..=1 {
                let x = (location.x as i32 + dx).rem_euclid(width as i32) as u16;
                let tile = Location::new(x, y as u16);
                for unit in &self.game.units {
                    if unit.location == tile && unit.owner() != owner {
                        contacts.push(unit.owner());
                    }
                }
                for city in &self.game.cities {
                    if city.location == tile && city.owner() != owner {
                        contacts.push(city.owner());
                    }
                }
            }
        }
        contacts.sort_unstable();
        contacts.dedup();
        for other in contacts {
            if self.game.have_met(owner, other) {
                continue;
            }
            self.game.make_peace(owner, other);
            self.events.push(Event::new(format!(
                "{:?} and {:?} meet for the first time",
                self.game.players[owner.index()].civilization,
                self.game.players[other.index()].civilization
            )));
        }
    }
}
