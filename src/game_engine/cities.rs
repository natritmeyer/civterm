use super::*;

use crate::game_engine::{Event, SettleError};
use crate::model::cartography::Location;
use crate::model::cities::{CityId, CityImprovement, ProductionTarget};
use crate::model::civilizations::PlayerId;
use crate::model::units::UnitId;

/// A city of the player's that finished building during the last round: which
/// city it was, what it finished, and the name to show for it. The TUI queues
/// one window per city once the round resolves back to the human, so it can
/// both announce the completion and open that city's window to set the next
/// order. The name is snapshotted here because a city's name outlives nothing —
/// the city itself may be gone by the time the window is answered.
#[derive(Clone, Debug, PartialEq)]
pub struct BuildComplete {
    pub city: CityId,
    pub city_name: String,
    pub target: ProductionTarget,
}

impl Engine {
    /// Drain the record of every city of the player's that finished building
    /// during the last round, in the order the cities were processed. The TUI
    /// walks this queue one window at a time.
    pub fn drain_build_completions(&mut self) -> Vec<BuildComplete> {
        std::mem::take(&mut self.builds)
    }
    pub(super) fn found_city(&mut self, unit: UnitId, name: String) {
        let (owner, location) = match self.ensure_can_found(unit) {
            Ok(legal) => legal,
            Err(SettleError::NoSuchUnit(_)) => {
                self.events.push(Event::new("No such unit"));
                return;
            }
            Err(error) => {
                self.events.push(Event::new(error.message()));
                return;
            }
        };
        self.game.remove_unit(unit);
        let city_id = self.game.add_city(owner, name.clone(), location);
        self.game.auto_assign_work(city_id);
        let first_city = self
            .game
            .cities
            .iter()
            .filter(|city| city.owner() == owner)
            .count()
            == 1;
        if first_city {
            self.game.begin_research(owner);
            let target = self.game.advancement_in_progress(owner).unwrap();
            self.events.push(Event::for_player(
                owner,
                format!(
                    "{:?} begin researching {:?}",
                    self.game.players[owner.index()].civilization,
                    target
                ),
            ));
        }
        self.game.reveal_tiles_surrounding_city_at(owner, location);
        self.events.push(Event::for_player(
            owner,
            format!("Unit {} founds {}", unit.index(), name),
        ));
    }
    pub(super) fn set_production(&mut self, city: CityId, target: ProductionTarget) {
        if !self
            .game
            .players
            .get(self.current_player_index.index())
            .is_some_and(|p| p.can_build(target))
        {
            let reason = match target.required_advancement() {
                Some(adv) => format!("requires {:?}", adv),
                None => "not available".to_string(),
            };
            self.events.push(Event::new(format!(
                "Cannot produce {:?}: {}",
                target, reason
            )));
            return;
        }
        match self
            .game
            .cities
            .iter_mut()
            .find(|c| c.id() == city && c.owner() == self.current_player_index)
        {
            Some(city) => {
                city.set_production(target);
                self.events.push(Event::for_player(
                    self.current_player_index,
                    format!("{} begins producing {:?}", city.name, target),
                ));
            }
            None => self.events.push(Event::new("No such city")),
        }
    }
    pub(super) fn process_cities(&mut self, owner: PlayerId) {
        let city_ids: Vec<CityId> = self
            .game
            .cities
            .iter()
            .filter(|c| c.owner() == owner)
            .map(|c| c.id())
            .collect();
        for city_id in city_ids {
            let city_name = self
                .game
                .cities
                .iter()
                .find(|c| c.id() == city_id)
                .unwrap()
                .name
                .clone();
            let result = self.game.process_city(city_id);
            if result.grew {
                self.game.auto_assign_work(city_id);
                self.events.push(Event::for_player(
                    owner,
                    format!(
                        "{} grows to size {}",
                        city_name,
                        self.game
                            .cities
                            .iter()
                            .find(|c| c.id() == city_id)
                            .unwrap()
                            .population()
                    ),
                ));
            }
            if let Some(target) = result.completed {
                // Only the human's completions are worth a window: a rival's
                // city finishing a unit is news, not a decision for the player.
                if owner == PlayerId::new(0) {
                    self.builds.push(BuildComplete {
                        city: city_id,
                        city_name: city_name.clone(),
                        target,
                    });
                }
                match target {
                    ProductionTarget::Unit(unit_class) => {
                        let city = self.game.cities.iter().find(|c| c.id() == city_id).unwrap();
                        let location = city.location;
                        let has_barracks = city.improvements().contains(&CityImprovement::Barracks);
                        let unit = self
                            .game
                            .spawn_unit(unit_class, location, owner, Some(city_id));
                        if has_barracks
                            && let Some(index) = self.game.units.iter().position(|u| u.id() == unit)
                        {
                            // A Barracks trains the city's recruits: they
                            // mustered as veterans before taking the field.
                            self.game.units[index].promote();
                        }
                        self.events.push(Event::for_player(
                            owner,
                            format!("{} produces {:?}", city_name, unit_class),
                        ));
                    }
                    ProductionTarget::Improvement(_) => {
                        self.events.push(Event::for_player(
                            owner,
                            format!("{} completes {:?}", city_name, target),
                        ));
                    }
                }
            }
            if result.starving {
                self.events.push(Event::for_player(
                    owner,
                    format!("{} is starving", city_name),
                ));
            }
        }
    }
    pub(super) fn ensure_can_found(
        &self,
        unit: UnitId,
    ) -> Result<(PlayerId, Location), SettleError> {
        let unit = self.owned_unit(unit).ok_or(SettleError::NoSuchUnit(unit))?;
        if !unit.unit_class.can_found_city() {
            return Err(SettleError::NotASettler(unit.id()));
        }
        if unit.is_transported() {
            return Err(SettleError::Transported(unit.id()));
        }
        let location = unit.location;
        if self.game.map.tile_at(location).terrain.is_water() {
            return Err(SettleError::LandRequired(unit.id()));
        }
        if self
            .game
            .cities
            .iter()
            .any(|city| city.location == location)
        {
            return Err(SettleError::CityAlreadyHere(location));
        }
        Ok((unit.owner(), location))
    }
}
