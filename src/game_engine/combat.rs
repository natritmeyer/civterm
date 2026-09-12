use super::*;

const HIT_POINTS: u32 = 10;

use crate::game_engine::Event;
use crate::model::cartography::Location;
use crate::model::cities::CityId;
use crate::model::civilizations::PlayerId;
use crate::model::geography::Terrain;
use crate::model::units::{Unit, UnitId};

impl Engine {
    pub(super) fn resolve_move_combat(&mut self, attacker: UnitId, tile: Location) {
        let attacker_idx = self
            .game
            .units
            .iter()
            .position(|u| u.id() == attacker)
            .expect("the moving unit exists");
        let defender_idx = self.select_defender(attacker_idx, tile);

        let attacker_id = self.game.units[attacker_idx].id();
        let defender_id = self.game.units[defender_idx].id();
        let attacker_owner = self.game.units[attacker_idx].owner();
        let defender_owner = self.game.units[defender_idx].owner();
        self.events.push(Event::new(format!(
            "Unit {} attacks Unit {}",
            attacker_id.index(),
            defender_id.index()
        )));

        let attacker_power = self.attacker_power(&self.game.units[attacker_idx]);
        let defender_power = self.defender_power(&self.game.units[defender_idx]);
        let attacker_won = self.resolve_combat(attacker_power, defender_power);

        if attacker_won {
            let was_veteran = self.game.units[attacker_idx].is_veteran();
            self.game.remove_unit(defender_id);
            let tile_is_clear = !self.game.units.iter().any(|unit| {
                unit.location == tile && self.game.at_war(attacker_owner, unit.owner())
            });
            // Advancing onto the cleared tile reveals the ring around it.
            if tile_is_clear {
                self.game.reveal_tiles_at(attacker_owner, tile);
            }
            let attacker_unit = self.owned_unit_mut(attacker_id).unwrap();
            if tile_is_clear {
                attacker_unit.location = tile;
            }
            attacker_unit.spend_turn();
            if !was_veteran {
                attacker_unit.promote();
            }
            self.events.push(Event::new(format!(
                "Unit {} defeats Unit {}",
                attacker_id.index(),
                defender_id.index()
            )));
            self.eliminate_if_annihilated(defender_owner);
        } else {
            self.game.remove_unit(attacker_id);
            self.events.push(Event::new(format!(
                "Unit {} repels Unit {}",
                defender_id.index(),
                attacker_id.index()
            )));
            self.eliminate_if_annihilated(attacker_owner);
        }
    }
    /// owner. Units homed to the captured city are disbanded. The attacking
    /// unit advances onto the city tile.
    pub(super) fn capture_city(&mut self, city_id: CityId, unit: UnitId, destination: Location) {
        let city_name = self
            .game
            .cities
            .iter()
            .find(|c| c.id() == city_id)
            .unwrap()
            .name
            .clone();
        let old_owner = self
            .game
            .cities
            .iter()
            .find(|c| c.id() == city_id)
            .unwrap()
            .owner();
        let disbanded = self.game.disband_units_homed_to(city_id);
        self.game
            .cities
            .iter_mut()
            .find(|c| c.id() == city_id)
            .unwrap()
            .change_owner(self.current_player_index);
        let mut_unit = self.owned_unit_mut(unit).unwrap();
        mut_unit.location = destination;
        mut_unit.spend_turn();
        self.game
            .reveal_tiles_at(self.current_player_index, destination);
        self.events.push(Event::new(format!(
            "{:?} capture {} (formerly {:?}'s)",
            self.game.players[self.current_player_index.index()].civilization,
            city_name,
            self.game.players[old_owner.index()].civilization
        )));
        if disbanded > 0 {
            self.events.push(Event::new(format!(
                "{} units disband with the loss of {}",
                disbanded, city_name
            )));
        }
        self.eliminate_if_cityless(old_owner);
    }
    /// A civilization that loses its last city is removed from play, even if
    /// stragglers remain in the field.
    pub(super) fn eliminate_if_cityless(&mut self, player: PlayerId) {
        if !self.owns_any_city(player) {
            self.eliminate(player);
        }
    }
    /// A civilization with no cities left is only removed once its last unit
    /// is gone too; a settler still in the field can yet found a new city.
    pub(super) fn eliminate_if_annihilated(&mut self, player: PlayerId) {
        if !self.owns_any_city(player) && !self.owns_any_unit(player) {
            self.eliminate(player);
        }
    }
    pub(super) fn owns_any_city(&self, player: PlayerId) -> bool {
        self.game.cities.iter().any(|city| city.owner() == player)
    }
    pub(super) fn owns_any_unit(&self, player: PlayerId) -> bool {
        self.game.units.iter().any(|unit| unit.owner() == player)
    }
    /// Remove `player` from play: mark them eliminated and disband whatever
    /// units they still have. Idempotent, so the many loss sites can call it
    /// freely.
    pub(super) fn eliminate(&mut self, player: PlayerId) {
        if self.game.players[player.index()].eliminated() {
            return;
        }
        self.game.players[player.index()].mark_eliminated();
        let civilization = self.game.players[player.index()].civilization;
        let disbanded = self.game.remove_units_owned_by(player);
        self.events
            .push(Event::new(format!("{civilization:?} has been eliminated")));
        if disbanded > 0 {
            self.events.push(Event::new(format!(
                "{disbanded} units disband with the loss of {civilization:?}"
            )));
        }
    }
    pub(super) fn select_defender(&self, attacker_idx: usize, tile: Location) -> usize {
        let owner = self.game.units[attacker_idx].owner();
        self.game
            .units
            .iter()
            .enumerate()
            .filter(|(_, unit)| {
                unit.location == tile
                    && unit.owner() != owner
                    && self.game.at_war(owner, unit.owner())
            })
            .max_by(|(_, a), (_, b)| {
                (self.defender_power(a), a.id().index())
                    .cmp(&(self.defender_power(b), b.id().index()))
            })
            .expect("the target's tile always holds at least the named enemy unit")
            .0
    }
    pub(super) fn attacker_power(&self, unit: &Unit) -> u32 {
        let base = unit.unit_class.attack() as u32 * 10;
        if unit.is_veteran() {
            base * 3 / 2
        } else {
            base
        }
    }
    pub(super) fn defender_power(&self, unit: &Unit) -> u32 {
        let base = unit.unit_class.defence() as u32 * 10;
        let mut power = base;
        if self.game.map.tile_at(unit.location).terrain == Terrain::Mountain {
            power *= 2;
        }
        let is_in_home_city = self
            .game
            .cities
            .iter()
            .any(|city| city.location == unit.location && city.owner() == unit.owner());
        if is_in_home_city {
            power = power * 3 / 2;
        }
        if unit.is_veteran() {
            power = power * 3 / 2;
        }
        power
    }
    pub(super) fn resolve_combat(&mut self, attacker_power: u32, defender_power: u32) -> bool {
        let total = attacker_power + defender_power;
        if total == 0 {
            return false;
        }
        let mut attacker_hp = HIT_POINTS;
        let mut defender_hp = HIT_POINTS;
        while attacker_hp > 0 && defender_hp > 0 {
            let hit = self.rng.in_range(total);
            if hit < attacker_power {
                defender_hp -= 1;
            } else {
                attacker_hp -= 1;
            }
        }
        defender_hp == 0
    }
}
