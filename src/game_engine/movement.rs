use super::*;

use crate::game_engine::{Event, MoveError};
use crate::model::cartography::{Direction, Location};
use crate::model::geography::TerrainImprovement;
use crate::model::units::{Unit, UnitClass, UnitId};

impl Engine {
    pub(super) fn move_unit(&mut self, unit: UnitId, direction: Direction) {
        let (destination, cost) = match self.ensure_can_move(unit, direction) {
            Ok(legal) => legal,
            Err(MoveError::NoSuchUnit(_)) => {
                self.events.push(Event::new("No such unit"));
                return;
            }
            Err(error) => {
                self.events.push(Event::new(error.message()));
                return;
            }
        };

        let owner = self.owned_unit(unit).unwrap().owner();
        self.meet_contacts_within(destination, owner);
        let enemies_present = self
            .game
            .units
            .iter()
            .any(|u| u.location == destination && self.game.at_war(owner, u.owner()));
        if enemies_present {
            self.resolve_move_combat(unit, destination);
            return;
        }

        // An at-war unit moving onto a foreign city tile that has no defending
        // units captures the city for the attacker.
        let capture_city = self.game.cities.iter().find(|c| {
            c.location == destination && c.owner() != owner && self.game.at_war(owner, c.owner())
        });
        if let Some(city) = capture_city {
            let defender_present = self.game.units.iter().any(|u| {
                u.location == destination && u.owner() == city.owner() && u.owner() != owner
            });
            if !defender_present {
                self.capture_city(city.id(), unit, destination);
                return;
            }
        }

        let mut_unit = self.owned_unit_mut(unit).unwrap();
        mut_unit.location = destination;
        mut_unit.spend_moves(cost);
        self.game.reveal_tiles_at(owner, destination);
        self.events.push(Event::new(format!(
            "Unit {} moves {:?}",
            unit.index(),
            direction
        )));
    }
    pub(super) fn ensure_medium_access(
        &self,
        unit: &Unit,
        destination: Location,
    ) -> Result<(), MoveError> {
        let tile_is_water = self.game.map.tile_at(destination).terrain.is_water();
        if unit.unit_class.can_travel_water() != tile_is_water {
            Err(MoveError::CannotCrossLandSeaBorder(unit.id()))
        } else {
            Ok(())
        }
    }
    pub(super) fn ensure_can_move(
        &self,
        unit: UnitId,
        direction: Direction,
    ) -> Result<(Location, u8), MoveError> {
        let unit = self.ensure_unit_owned(unit)?;
        self.ensure_moves_remaining(unit)?;
        let destination = self.ensure_destination_on_map(unit.location, direction)?;
        self.ensure_medium_access(unit, destination)?;
        self.ensure_peaceful_passage(unit, destination)?;
        // The move is made first and the cost is then deducted from the
        // budget, so a unit only needs *some* movement left, not a full
        // tile's worth. A slow unit enters dense/mountain terrain but is
        // exhausted by it; a fast unit is slowed to one tile per move.
        let cost = self.game.map.tile_at(destination).terrain.movement_cost();
        Ok((destination, cost))
    }
    pub(super) fn ensure_peaceful_passage(
        &self,
        unit: &Unit,
        destination: Location,
    ) -> Result<(), MoveError> {
        let owner = unit.owner();
        let mut has_foreign_occupant = false;
        let mut has_enemy_occupant = false;
        for occupied in self
            .game
            .units
            .iter()
            .filter(|u| u.location == destination && u.owner() != owner)
        {
            has_foreign_occupant = true;
            if self.game.at_war(owner, occupied.owner()) {
                has_enemy_occupant = true;
            }
        }
        // A foreign city on the destination tile blocks passage unless the two
        // civilisations are at war.
        if let Some(city) = self
            .game
            .cities
            .iter()
            .find(|c| c.location == destination && c.owner() != owner)
        {
            has_foreign_occupant = true;
            if self.game.at_war(owner, city.owner()) {
                has_enemy_occupant = true;
            }
        }
        if has_foreign_occupant && !has_enemy_occupant {
            Err(MoveError::PeacefulTileOccupied(unit.id()))
        } else {
            Ok(())
        }
    }
    pub(super) fn ensure_unit_owned(&self, unit: UnitId) -> Result<&Unit, MoveError> {
        self.owned_unit(unit).ok_or(MoveError::NoSuchUnit(unit))
    }
    pub(super) fn ensure_moves_remaining(&self, unit: &Unit) -> Result<(), MoveError> {
        if unit.moves_remaining() > 0 {
            Ok(())
        } else {
            Err(MoveError::NoMovesRemaining(unit.id()))
        }
    }
    pub(super) fn ensure_destination_on_map(
        &self,
        from: Location,
        direction: Direction,
    ) -> Result<Location, MoveError> {
        self.game
            .map
            .destination(from, direction)
            .ok_or(MoveError::CannotMoveThere)
    }
    pub(super) fn fortify(&mut self, unit: UnitId) {
        match self.owned_unit_mut(unit) {
            Some(u) => {
                u.fortify();
                u.spend_turn();
                self.events
                    .push(Event::new(format!("Unit {} fortifies", unit.index())));
            }
            None => self.events.push(Event::new("No such unit")),
        }
    }
    pub(super) fn sentry(&mut self, unit: UnitId) {
        match self.owned_unit_mut(unit) {
            Some(u) => {
                u.sentry();
                u.spend_turn();
                self.events
                    .push(Event::new(format!("Unit {} stands sentry", unit.index())));
            }
            None => self.events.push(Event::new("No such unit")),
        }
    }
    pub(super) fn work(&mut self, unit: UnitId, improvement: TerrainImprovement) {
        let location = match self.owned_unit(unit) {
            Some(u) => {
                if u.unit_class != UnitClass::Settler {
                    self.events
                        .push(Event::new("Only settlers can build improvements"));
                    return;
                }
                u.location
            }
            None => {
                self.events.push(Event::new("No such unit"));
                return;
            }
        };
        if !self.game.map.tile_at(location).can_build(improvement) {
            self.events
                .push(Event::new(format!("Cannot build {:?} here", improvement)));
            return;
        }
        if let Some(u) = self.owned_unit_mut(unit) {
            u.work(improvement);
            u.spend_turn();
        }
        self.events.push(Event::new(format!(
            "Unit {} begins building {:?}",
            unit.index(),
            improvement
        )));
    }
    pub(super) fn cancel_order(&mut self, unit: UnitId) {
        match self.owned_unit_mut(unit) {
            Some(u) => {
                u.cancel_order();
                u.spend_turn();
                self.events
                    .push(Event::new(format!("Unit {} order cancelled", unit.index())));
            }
            None => self.events.push(Event::new("No such unit")),
        }
    }
    pub(super) fn owned_unit(&self, unit: UnitId) -> Option<&Unit> {
        self.game
            .units
            .iter()
            .find(|u| u.id() == unit && u.owner() == self.current_player_index)
    }
    pub(super) fn owned_unit_mut(&mut self, unit: UnitId) -> Option<&mut Unit> {
        let unit_ref = self.game.units.iter_mut().find(|u| u.id() == unit)?;
        if unit_ref.owner() == self.current_player_index {
            Some(unit_ref)
        } else {
            None
        }
    }
}
