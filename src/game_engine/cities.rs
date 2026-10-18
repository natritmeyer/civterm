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

/// A city of the player's that lost a citizen to starvation during the last
/// round: the name to show for it. The TUI announces it with a window once the
/// round resolves back to the human, so the player learns which city is
/// hungry. The name is snapshotted because the city itself may be gone by the
/// time the window is answered.
#[derive(Clone, Debug, PartialEq)]
pub struct StarvationNotice {
    pub city_name: String,
}

/// A round in which a civilization could not pay its improvements' upkeep: the
/// total maintenance owed and the unit disbanded to cover it, if any was left
/// to take. The TUI announces the human's with a window once the round
/// resolves, so the player learns why a unit vanished and the treasury is
/// empty; a rival's is news, not a decision for the player.
#[derive(Clone, Debug, PartialEq)]
pub struct UpkeepNotice {
    pub upkeep: u32,
    pub disbanded: Option<String>,
}

impl Engine {
    /// Drain the record of every city of the player's that starved during the
    /// last round, in the order the cities were processed. The TUI walks this
    /// queue one window at a time.
    pub fn drain_starvations(&mut self) -> Vec<StarvationNotice> {
        std::mem::take(&mut self.starvations)
    }

    /// Drain the record of every city of the player's that finished building
    /// during the last round, in the order the cities were processed. The TUI
    /// walks this queue one window at a time.
    pub fn drain_build_completions(&mut self) -> Vec<BuildComplete> {
        std::mem::take(&mut self.builds)
    }

    /// Drain the record of every upkeep shortfall from the last round, in the
    /// order the owners were processed. Only the human's are recorded; the TUI
    /// walks this queue one window at a time.
    pub fn drain_upkeep_notices(&mut self) -> Vec<UpkeepNotice> {
        std::mem::take(&mut self.upkeeps)
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
        let owner = self.current_player_index;
        if !self
            .game
            .players
            .get(owner.index())
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
        let location = match self
            .game
            .cities
            .iter()
            .find(|c| c.id() == city && c.owner() == owner)
        {
            Some(city) => city.location,
            None => {
                self.events.push(Event::new("No such city"));
                return;
            }
        };
        // A city away from the sea cannot launch a ship: with no water one step
        // away the vessel would be stranded on land the moment it was built.
        if matches!(target, ProductionTarget::Unit(class) if class.can_travel_water())
            && !self.game.borders_water(location)
        {
            self.events.push(Event::new(format!(
                "Cannot produce {:?}: the city is not on the coast",
                target
            )));
            return;
        }
        match self
            .game
            .cities
            .iter_mut()
            .find(|c| c.id() == city && c.owner() == owner)
        {
            Some(city) => {
                city.set_production(target);
                self.events.push(Event::for_player(
                    owner,
                    format!("{} begins producing {:?}", city.name, target),
                ));
            }
            None => self.events.push(Event::new("No such city")),
        }
    }
    /// Rush the current build of `city` for gold, completing it at once. The
    /// cost is twice the shields still needed, never less than one; a city the
    /// player does not own, an idle city, or a treasury too thin to pay is
    /// refused with an event. A rushed unit is mustered immediately, wearing a
    /// Barracks' veteran stripe like any other.
    pub(super) fn buy_production(&mut self, city: CityId) {
        let owner = self.current_player_index;
        let Some(index) = self
            .game
            .cities
            .iter()
            .position(|c| c.id() == city && c.owner() == owner)
        else {
            self.events.push(Event::new("No such city"));
            return;
        };
        let Some(cost) = self.game.cities[index].rush_cost() else {
            self.events
                .push(Event::new("The city is building nothing to rush"));
            return;
        };
        let gold = self.game.players[owner.index()].gold();
        if gold < cost {
            self.events.push(Event::new(format!(
                "Cannot rush: {cost} gold needed, {gold} in the treasury"
            )));
            return;
        }
        self.game.players[owner.index()].spend_gold(cost);
        let city_name = self.game.cities[index].name.clone();
        let target = self.game.cities[index]
            .rush_production()
            .expect("a city with a rush cost has a build to complete");
        match target {
            ProductionTarget::Unit(class) => {
                let location = self.game.cities[index].location;
                let veteran = self.game.cities[index]
                    .improvements()
                    .contains(&CityImprovement::Barracks);
                let spawned = self.game.spawn_unit(class, location, owner, Some(city));
                if veteran
                    && let Some(position) = self.game.units.iter().position(|u| u.id() == spawned)
                {
                    self.game.units[position].promote();
                }
                self.events.push(Event::for_player(
                    owner,
                    format!("{city_name} rushes {class:?}"),
                ));
            }
            ProductionTarget::Improvement(_) => {
                self.events.push(Event::for_player(
                    owner,
                    format!("{city_name} rushes {target:?}"),
                ));
            }
        }
    }
    pub(super) fn process_cities(&mut self, owner: PlayerId) {
        let scales = self.economy_scales(owner);
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
            let result = self.game.process_city(city_id, scales);
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
            // A city that actually lost a citizen is announced by a window, and
            // only the human's own counts: a rival's hungry city is news, not a
            // decision for the player. A size-one city that cannot lose its last
            // citizen gets the log line but no window, because no population was
            // lost to report.
            if result.lost_citizen && owner == PlayerId::new(0) {
                self.starvations.push(StarvationNotice {
                    city_name: city_name.clone(),
                });
            }
        }
        // Trade routes pay every owner that holds an endpoint, once a round,
        // before the upkeep bill is settled so a route can cover its cities'
        // upkeep.
        let trade_income = self.game.trade_income(owner);
        if trade_income > 0 {
            self.game.players[owner.index()].add_gold(trade_income);
            self.events.push(Event::for_player(
                owner,
                format!("Trade routes earn {trade_income} gold"),
            ));
        }
        self.charge_upkeep(owner);
    }

    /// Charge `owner` the maintenance of every city it holds. The treasury
    /// pays what it can; a shortfall clamps it at zero and disbands the
    /// costliest unit, queuing a notice for the human. Rivals are punished the
    /// same way but never announce it.
    fn charge_upkeep(&mut self, owner: PlayerId) {
        let upkeep: u32 = self
            .game
            .cities
            .iter()
            .filter(|city| city.owner() == owner)
            .map(|city| city.upkeep())
            .sum();
        if upkeep == 0 {
            return;
        }
        if self.game.players[owner.index()].spend_gold(upkeep) {
            return;
        }
        self.game.players[owner.index()].set_gold(0);
        let disbanded = self.disband_costliest_unit(owner);
        self.events.push(Event::for_player(
            owner,
            match &disbanded {
                Some(name) => format!("Upkeep {upkeep} gold unpaid; a {name} disbands"),
                None => format!("Upkeep {upkeep} gold unpaid; the treasury is empty"),
            },
        ));
        if owner == PlayerId::new(0) {
            self.upkeeps.push(UpkeepNotice { upkeep, disbanded });
        }
    }

    /// Disband the costliest unit `owner` has in the field, returning its class
    /// name. Ties break on the highest unit id, so the choice is deterministic.
    /// A disbanded carrier takes its cargo down with it.
    fn disband_costliest_unit(&mut self, owner: PlayerId) -> Option<String> {
        let victim = self
            .game
            .units
            .iter()
            .filter(|unit| unit.owner() == owner)
            .max_by_key(|unit| (unit.unit_class.resource_cost(), unit.id().index()))
            .map(|unit| unit.id())?;
        let name = self
            .game
            .units
            .iter()
            .find(|unit| unit.id() == victim)
            .map(|unit| format!("{:?}", unit.unit_class));
        self.game.remove_unit(victim);
        self.game.disband_cargo_of(victim);
        name
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
