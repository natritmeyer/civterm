use super::*;

use crate::model::advancements::Advancement;
use crate::model::cartography::Location;
use crate::model::cities::{City, CityId, CityImprovement};
use crate::model::civilizations::PlayerId;
use crate::model::units::{UnitClass, UnitId};

/// What a diplomat costs to get one of his city's secrets, or to take the city
/// itself. These are tuning numbers, so they live here as named constants
/// rather than as bare literals buried in a match arm.
pub const INVESTIGATE_CITY_COST: u32 = 25;
pub const STEAL_TECHNOLOGY_COST: u32 = 50;
pub const INDUSTRIAL_SABOTAGE_COST: u32 = 50;
pub const INCITE_REVOLT_COST: u32 = 100;
/// The per-head price of Subvert City: taking a city of its own people is
/// charged by the size of the prize, so a city of two costs twice a city of
/// one. The flat price a single-population city always paid, kept as the unit
/// so a small city still costs what it used to. The difference from Incite
/// remains the war it saves you from having to declare.
pub const SUBVERT_CITY_COST: u32 = 200;

/// The five things a diplomat may do once he is standing inside a rival city.
///
/// Two are ruled out by design. **Establish Embassy** is absent because it
/// turns a palace into an embassy, and this game has no governments; the
/// embassy would be a flag with nothing behind it. **Meet with King** is
/// absent because a first meeting is free and automatic — `meet_contacts_within`
/// makes peace the moment the diplomat is in sight of anyone — so charging a
/// turn to say what the walk in already said would be a tax, not a choice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiplomatAction {
    /// Look inside the city and learn what it is building.
    InvestigateCity,
    /// Take one of the rival's advances for nothing.
    StealTechnology,
    /// Tear out one of the city's buildings.
    IndustrialSabotage,
    /// Buy the garrison's loyalty. This one declares war.
    InciteRevolt,
    /// Take the city outright, without a war. Only gold is required — no
    /// soldiers need be standing in it — and the bill grows with the city's
    /// population.
    SubvertCity,
}

impl DiplomatAction {
    /// Every action, in the order the window lists them. The order is fixed so
    /// the list never reshuffles under the player's cursor.
    pub const ALL: [DiplomatAction; 5] = [
        DiplomatAction::InvestigateCity,
        DiplomatAction::StealTechnology,
        DiplomatAction::IndustrialSabotage,
        DiplomatAction::InciteRevolt,
        DiplomatAction::SubvertCity,
    ];

    /// What the action charges. Only Subvert City reads `city`: its price is
    /// per head of population, so the window and the charge both take the city
    /// the diplomat is standing in. `None` prices it for a city of one — the
    /// smallest city there is, and the only case where it needs no context.
    pub fn cost(self, city: Option<&City>) -> u32 {
        match self {
            DiplomatAction::InvestigateCity => INVESTIGATE_CITY_COST,
            DiplomatAction::StealTechnology => STEAL_TECHNOLOGY_COST,
            DiplomatAction::IndustrialSabotage => INDUSTRIAL_SABOTAGE_COST,
            DiplomatAction::InciteRevolt => INCITE_REVOLT_COST,
            DiplomatAction::SubvertCity => {
                SUBVERT_CITY_COST.saturating_mul(city.map_or(1, |c| c.population()))
            }
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            DiplomatAction::InvestigateCity => "Investigate City",
            DiplomatAction::StealTechnology => "Steal Technology",
            DiplomatAction::IndustrialSabotage => "Industrial Sabotage",
            DiplomatAction::InciteRevolt => "Incite a Revolt",
            DiplomatAction::SubvertCity => "Subvert City",
        }
    }
}

/// One row of the diplomat's window: what it would do, what it costs, and why
/// it cannot be done right now. `blocked` is the refusal the player reads, or
/// `None` when the action is theirs to take.
#[derive(Clone, Debug, PartialEq)]
pub struct DiplomatOption {
    pub action: DiplomatAction,
    pub cost: u32,
    pub blocked: Option<String>,
}

/// A diplomat standing in a rival city, reported to the TUI so it can offer
/// the choices. The city's name is snapshotted for the same reason a build
/// completion snapshots its own: the city may be gone by the time the window is
/// answered, and Subvert City alone can destroy it out from under the player.
/// The tile the walk-in left and the move budget it carried are carried too,
/// so a dismissal can put the diplomat back exactly as he was. Never persisted
/// — the record exists only to open a window.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiplomatAudience {
    pub unit: UnitId,
    pub city: CityId,
    pub city_name: String,
    pub from: Location,
    pub moves_before: u8,
}

/// What a Steal Technology action produced, for the TUI to announce: either
/// the advance that came away and the rival it came from, or the refusal — the
/// rival knew nothing the player could take, the attempt cost nothing, and the
/// diplomat walks back out. The rival's display name is snapshotted because it
/// is whom the window must name, and the name it carried at the moment of the
/// attempt is the one the window says. Never persisted: the record exists only
/// to open a window.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StealOutcome {
    /// An advance came away: which one, and whose it was.
    Stolen {
        advancement: Advancement,
        rival: String,
    },
    /// There was nothing worth taking: the report says whose cupboard it was.
    NothingToSteal { rival: String },
}

/// What an Industrial Sabotage action destroyed, for the TUI to announce: the
/// improvement that came down and the city it stood in, by display name so the
/// window can say what was lost. The name is snapshotted like a build
/// completion's — the city is where the damage happened, and the name it
/// carried at the moment of the sabotage is the one the window says. Never
/// persisted: the record exists only to open a window.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SabotageNotice {
    pub improvement: CityImprovement,
    pub city_name: String,
}

/// Every way a diplomatic action can be refused. The `message` is the only way
/// the refusal reaches the player, and the window's disabled rows and the
/// engine's enforcement both come from `DiplomatError::message`, so the row the
/// player can read and the rule the engine enforces cannot drift apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiplomatError {
    NoSuchUnit(UnitId),
    NotADiplomat(UnitId),
    NotInForeignCity(UnitId),
    InsufficientGold { required: u32, held: u32 },
    NoImprovementsToDestroy,
    NoGarrisonToIncite,
}

impl DiplomatError {
    pub fn message(&self) -> String {
        match self {
            DiplomatError::NoSuchUnit(_) => "No such unit".to_string(),
            DiplomatError::NotADiplomat(unit) => {
                format!("Unit {} is not a diplomat", unit.index())
            }
            DiplomatError::NotInForeignCity(unit) => {
                format!("Unit {} is not inside a rival city", unit.index())
            }
            DiplomatError::InsufficientGold { required, held } => {
                format!("Not enough gold: {required} needed, {held} held")
            }
            DiplomatError::NoImprovementsToDestroy => {
                "The city has nothing left to destroy".to_string()
            }
            DiplomatError::NoGarrisonToIncite => "The city holds no units to incite".to_string(),
        }
    }
}

impl Engine {
    /// Report the player's diplomat standing in a rival city, if that is where
    /// this unit is. The TUI drains this after a move and opens the window.
    pub fn drain_diplomat_audiences(&mut self) -> Vec<DiplomatAudience> {
        std::mem::take(&mut self.audiences)
    }

    /// The rival city the player's diplomat investigated by the action just
    /// submitted, if that is what it was. The TUI drains this to open the
    /// city's window once, as a read-only report that is not openable again.
    pub fn drain_investigation(&mut self) -> Option<CityId> {
        self.investigation.take()
    }

    /// What the player's diplomat's Steal Technology action just produced: the
    /// advance he took, or the refusal when the rival knew nothing worth
    /// taking. The TUI drains it to announce it in a window of its own; the
    /// record is transient and never persisted.
    pub fn drain_steal_outcome(&mut self) -> Option<StealOutcome> {
        self.steal.take()
    }

    /// What the player's diplomat's Industrial Sabotage action just destroyed,
    /// if that is what it was: the improvement and the city it stood in. The
    /// TUI drains it to announce it in a window of its own; the record is
    /// transient and never persisted.
    pub fn drain_sabotage_notice(&mut self) -> Option<SabotageNotice> {
        self.sabotage.take()
    }

    /// The five actions on offer to `unit` where it stands, each with its cost
    /// and, where it cannot be taken, the reason it cannot.
    pub fn diplomat_options(&self, unit: UnitId) -> Vec<DiplomatOption> {
        let city = self
            .diplomat_in_rival_city(unit)
            .ok()
            .map(|(_, c)| c)
            .and_then(|c| self.city_opt(c));
        DiplomatAction::ALL
            .into_iter()
            .map(|action| DiplomatOption {
                action,
                cost: action.cost(city),
                blocked: self
                    .diplomat_blocker(action, unit)
                    .err()
                    .map(|error| error.message()),
            })
            .collect()
    }

    /// The first reason `action` cannot be performed by `unit` where it stands.
    /// This is the single source of the rule: the window reads it to grey out a
    /// row, and `perform_diplomat_action` enforces it again, so a stale window
    /// cannot buy something the rules no longer allow.
    pub(super) fn diplomat_blocker(
        &self,
        action: DiplomatAction,
        unit: UnitId,
    ) -> Result<(), DiplomatError> {
        let (owner, city_id) = self.diplomat_in_rival_city(unit)?;
        let city = self.city(city_id);
        let held = self.game.players[owner.index()].gold();
        let cost = action.cost(Some(city));
        if held < cost {
            return Err(DiplomatError::InsufficientGold {
                required: cost,
                held,
            });
        }
        match action {
            // A look is always for sale: the gold check above is the whole
            // of the rule, and a city can be investigated again and again.
            DiplomatAction::InvestigateCity => {}
            // Stealing is always actionable even when the rival knows nothing
            // the player could take: an empty cupboard is a refusal, not a
            // dead row, and the refusal sells itself (the attempt costs
            // nothing and the diplomat walks back out).
            DiplomatAction::StealTechnology => {}
            DiplomatAction::IndustrialSabotage => {
                if self.city(city_id).improvements().is_empty() {
                    return Err(DiplomatError::NoImprovementsToDestroy);
                }
            }
            DiplomatAction::InciteRevolt => {
                self.ensure_garrison_to_incite(city_id)?;
            }
            // Subverting needs only the gold, priced per head of population
            // above: the city is bought from under its people, not from a
            // garrison, so nobody has to be standing in it.
            DiplomatAction::SubvertCity => {}
        }
        Ok(())
    }

    /// The diplomat and the city he is standing in, or why there is no such
    /// pair. Every action starts here, so a unit that is not a diplomat, or is
    /// not inside a rival's city, cannot act in one.
    fn diplomat_in_rival_city(&self, unit: UnitId) -> Result<(PlayerId, CityId), DiplomatError> {
        let unit = self
            .owned_unit(unit)
            .ok_or(DiplomatError::NoSuchUnit(unit))?;
        if unit.unit_class != UnitClass::Diplomat {
            return Err(DiplomatError::NotADiplomat(unit.id()));
        }
        let owner = unit.owner();
        let city = self
            .game
            .cities
            .iter()
            .find(|c| c.location == unit.location && c.owner() != owner)
            .ok_or(DiplomatError::NotInForeignCity(unit.id()))?;
        Ok((owner, city.id()))
    }

    /// Inciting buys the garrison's own loyalty, so there must be a garrison
    /// to buy: a city with no soldiers of its own standing in it holds nothing
    /// to bribe. Subverting needs nothing but the gold, which is why it has no
    /// such guard. The diplomat's own side is no part of the deal — he acts
    /// alone, so no unit of the player's is ever required.
    fn ensure_garrison_to_incite(&self, city_id: CityId) -> Result<(), DiplomatError> {
        let city = self.city(city_id);
        let location = city.location;
        let garrison = self.units_standing_at(location, city.owner());
        if garrison.is_empty() {
            return Err(DiplomatError::NoGarrisonToIncite);
        }
        Ok(())
    }

    /// The advances the rival in `city_id` knows and the player does not — the
    /// whole cupboard. Which of them is actually takeable is the pick's own
    /// business (the research target first, then what the player could
    /// research next); this is only the range to choose from.
    fn advances_to_steal(&self, owner: PlayerId, city_id: CityId) -> Vec<Advancement> {
        let rival = self.city(city_id).owner();
        self.game.players[rival.index()]
            .advances_made()
            .iter()
            .copied()
            .filter(|advancement| !self.game.players[owner.index()].has_advancement(*advancement))
            .collect()
    }

    fn city(&self, city_id: CityId) -> &City {
        self.city_opt(city_id).expect("the city exists")
    }

    fn city_opt(&self, city_id: CityId) -> Option<&City> {
        self.game.cities.iter().find(|city| city.id() == city_id)
    }

    fn units_standing_at(&self, location: Location, owner: PlayerId) -> Vec<UnitId> {
        self.game
            .units
            .iter()
            .filter(|unit| {
                unit.location == location && unit.owner() == owner && !unit.is_transported()
            })
            .map(|unit| unit.id())
            .collect()
    }

    /// Perform `action` with the diplomat at `unit`. Every action consumes the
    /// diplomat and its gold — except a theft that comes away empty-handed,
    /// which is no purchase at all: nothing is spent and the diplomat walks
    /// back out, exactly as a dismissal would. The refusal is reported by its
    /// own window, so the that-walk remains the report's whole price.
    pub(super) fn perform_diplomat_action(&mut self, unit: UnitId, action: DiplomatAction) {
        if let Err(error) = self.diplomat_blocker(action, unit) {
            self.events.push(Event::new(error.message()));
            return;
        }
        let cost = action.cost(Some(
            self.city(self.diplomat_in_rival_city(unit).unwrap().1),
        ));
        let city_name = self
            .city(self.diplomat_in_rival_city(unit).unwrap().1)
            .name
            .clone();
        // Whether the action consumes the diplomat and its gold: true for
        // every purchase, false only for a theft that took nothing.
        let (message, spent) = match action {
            DiplomatAction::InvestigateCity => {
                let building = self
                    .city(self.diplomat_in_rival_city(unit).unwrap().1)
                    .production_target()
                    .map(|target| format!("{target:?}"))
                    .unwrap_or_else(|| "nothing in particular".to_string());
                self.city_mut(self.diplomat_in_rival_city(unit).unwrap().1)
                    .mark_investigated();
                let city_id = self.diplomat_in_rival_city(unit).unwrap().1;
                self.investigation = Some(city_id);
                (
                    format!(
                        "Unit {} investigates {city_name}: it is building {building}",
                        unit.index()
                    ),
                    true,
                )
            }
            DiplomatAction::StealTechnology => {
                let city_id = self.diplomat_in_rival_city(unit).unwrap().1;
                let choices = self.advances_to_steal(self.current_player_index, city_id);
                // The pick is not luck but priority: the player's current
                // research target if the rival knows it, otherwise the priciest
                // advance the player could begin researching next. An advance
                // the player cannot yet work on is no prize, so a rival whose
                // cupboard holds nothing researchable is robbed of nothing — the
                // theft comes away empty-handed, spends no gold, and (below) the
                // diplomat is handled by the refusal rather than spent.
                let stolen = {
                    let player = &self.game.players[self.current_player_index.index()];
                    match self.game.advancement_in_progress(self.current_player_index) {
                        Some(target) if choices.contains(&target) => Some(target),
                        _ => {
                            let researchable = player.researchable_advancements();
                            choices
                                .iter()
                                .copied()
                                .filter(|adv| researchable.contains(adv))
                                .max_by_key(|adv| adv.cost())
                        }
                    }
                };
                let rival = self.game.players[self.city(city_id).owner().index()]
                    .civilization
                    .display_name()
                    .to_string();
                match stolen {
                    Some(stolen) => {
                        self.game.players[self.current_player_index.index()]
                            .add_advancement(stolen);
                        // What came away is announced by its own window, so the
                        // TUI drains the record just as it drains an investigation.
                        self.steal = Some(StealOutcome::Stolen {
                            advancement: stolen,
                            rival,
                        });
                        (
                            format!("Unit {} steals {:?} from {city_name}", unit.index(), stolen),
                            true,
                        )
                    }
                    None => {
                        self.steal = Some(StealOutcome::NothingToSteal { rival });
                        (
                            format!(
                                "Unit {} finds nothing left to steal from {city_name}",
                                unit.index()
                            ),
                            false,
                        )
                    }
                }
            }
            DiplomatAction::IndustrialSabotage => {
                let city_id = self.diplomat_in_rival_city(unit).unwrap().1;
                // Walls first, as a catapult would: they are the one building
                // that is a defence in its own right, so tearing them down is
                // the sabotage worth paying for.
                let improvements = self.city(city_id).improvements().to_vec();
                let victim = if improvements.contains(&CityImprovement::CityWalls) {
                    CityImprovement::CityWalls
                } else {
                    let index = self.rng.in_range(improvements.len() as u32) as usize;
                    improvements[index]
                };
                self.city_mut(city_id).remove_improvement(victim);
                // What came down is announced by its own window, so the TUI
                // drains the record just as it drains a stolen technology.
                self.sabotage = Some(SabotageNotice {
                    improvement: victim,
                    city_name: city_name.clone(),
                });
                (
                    format!("Unit {} sabotages {victim:?} in {city_name}", unit.index()),
                    true,
                )
            }
            DiplomatAction::InciteRevolt => {
                let city_id = self.diplomat_in_rival_city(unit).unwrap().1;
                let city = self.city(city_id);
                let location = city.location;
                let rival = city.owner();
                // The bought soldiers are homed to the diplomat's own city:
                // he paid for them, they answer to his civ, and a home in the
                // city still held against the rival would die with it — the
                // disband sweep at a city's fall reads `home_city` and never
                // asks which side owns the unit.
                let home = self.owned_unit(unit).and_then(|buyer| buyer.home_city());
                let defected = self.units_standing_at(location, rival).len();
                if !self.game.at_war(self.current_player_index, rival) {
                    self.game.declare_war(self.current_player_index, rival);
                }
                for id in self.units_standing_at(location, rival) {
                    let bought = self
                        .game
                        .units
                        .iter_mut()
                        .find(|u| u.id() == id)
                        .expect("the unit is in the city");
                    bought.defect_to(self.current_player_index, home);
                    // A garrison is a fortify order with its turn spent, which
                    // is not the state the player's units are bought in.
                    // Cancelling the order and handing back the budget is
                    // what puts it into this turn's cycle of commandable
                    // units — and its first duty, while it still stands on
                    // the rival's own square, is to walk off it: every order
                    // refuses it there until it does.
                    bought.cancel_order();
                    bought.restore_moves();
                }
                (format!("{defected} units in {city_name} defect"), true)
            }
            DiplomatAction::SubvertCity => {
                let city_id = self.diplomat_in_rival_city(unit).unwrap().1;
                let location = self.city(city_id).location;
                let rival = self.city(city_id).owner();
                // The city changes hands without a shot, but the garrison
                // standing in it does not: it is the thing being taken away
                // from them, so it goes with the city.
                for id in self.units_standing_at(location, rival) {
                    self.game.remove_unit(id);
                }
                self.take_captured_city(city_id);
                (format!("Unit {} subverts {city_name}", unit.index()), true)
            }
        };
        if spent {
            self.game.players[self.current_player_index.index()].spend_gold(cost);
            self.game.remove_unit(unit);
        }
        self.events
            .push(Event::for_player(self.current_player_index, message));
    }

    /// Undo the step that carried `unit` into a rival city: the player turned
    /// down every offer, so the diplomat walks back onto `from`, the tile he
    /// stepped off, with his budget restored to `moves_before` — the dismissal
    /// costs no gold, spends no movement, and strands no one. This is the one
    /// landing that neither spends the terrain cost nor records a rival step:
    /// it is an undo, not an advance, so the plain-move tail does not apply.
    /// A unit that is no longer a diplomat inside a foreign city (a rival
    /// walked in and took it, say) is left alone.
    pub(super) fn withdraw_diplomat(&mut self, unit: UnitId, from: Location, moves_before: u8) {
        let Some(found) = self.owned_unit(unit) else {
            return;
        };
        if found.unit_class != UnitClass::Diplomat {
            return;
        }
        let owner = found.owner();
        let in_foreign_city = self
            .game
            .cities
            .iter()
            .any(|c| c.location == found.location && c.owner() != owner);
        if !in_foreign_city {
            return;
        }
        let diplomat = self
            .game
            .units
            .iter_mut()
            .find(|u| u.id() == unit)
            .expect("the diplomat found a moment ago still exists");
        diplomat.location = from;
        diplomat.restore_moves_to(moves_before);
        // The origin is already revealed from the walk-in, so this redraw is a
        // no-op for the player — it lands the unit the same way every other
        // landing does.
        self.game.reveal_tiles_at(owner, from);
    }

    fn city_mut(&mut self, city_id: CityId) -> &mut City {
        self.game
            .cities
            .iter_mut()
            .find(|city| city.id() == city_id)
            .expect("the city exists")
    }

    /// Report a diplomat that has just walked into a rival city, so the TUI can
    /// offer him a choice. Only the human's diplomat earns a window: a rival
    /// sending one is its own business, not a decision for the player. The
    /// tile the walk-in left and the budget it carried go into the record, so
    /// a dismissal can put the diplomat back exactly as he was.
    pub(super) fn record_diplomat_audience(
        &mut self,
        unit: UnitId,
        from: Location,
        destination: Location,
        moves_before: u8,
    ) {
        let owner = self.current_player_index;
        if owner != PlayerId::new(0) {
            return;
        }
        let Some(found) = self.game.units.iter().find(|u| u.id() == unit) else {
            return;
        };
        if found.unit_class != UnitClass::Diplomat {
            return;
        }
        let Some(city) = self
            .game
            .cities
            .iter()
            .find(|c| c.location == destination && c.owner() != owner)
        else {
            return;
        };
        self.audiences.push(DiplomatAudience {
            unit,
            city: city.id(),
            city_name: city.name.clone(),
            from,
            moves_before,
        });
    }
}
