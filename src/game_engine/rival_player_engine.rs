use super::*;

use crate::game_engine::GameView;
use crate::model::advancements::Advancement;
use crate::model::cartography::{Direction, Location};
use crate::model::cities::{City, CityId, ProductionTarget};
use crate::model::civilizations::PlayerId;
use crate::model::geography::Terrain;
use crate::model::units::{Unit, UnitClass, UnitId, UnitOrder};
use strum::IntoEnumIterator;

/// How many cities a rival civilization aims to grow before it stops producing
/// settlers. Modest so a rival fields a manageable, defensible empire instead
/// of blanketing the map.
const RIVAL_EXPANSION_TARGET_CITIES: usize = 4;

/// How many working-grid tiles a rival may let a new city share with the union
/// of every civilization's 21-tile working grids before the site counts as
/// genuinely open. Two cities sharing no more tiles than this sit at least
/// Chebyshev 4 apart and never poach each other's harvest, so the AI keeps its
/// cities a full grid apart wherever such a site is visible.
const CITY_FOOTPRINT_MAX_OVERLAP: usize = 3;

/// How many working-grid tiles a site may share with the union before it stops
/// counting as decent at all. A site sharing no more than this sits at least
/// Chebyshev 3 from every existing city — close enough to exploit the same
/// neighbourhood, never jammed against a grid. This tier exists because a
/// founding's own footprint is about all it can see: right after a city is
/// founded, no visible land is "open", only decent, so without this tier the
/// settler would either kneel over and found inside the crowd or march forever.
/// The best decent site is chosen before any frontier-push.
const CITY_FOOTPRINT_ACCEPTABLE_OVERLAP: usize = 6;

/// The rival's preferred research ladder, cheapest useful goals first so a
/// rival fields Bronze-gated troops early and then pursues its economy. When
/// none of these are researchable yet, the rival falls back to the cheapest
/// open advancement.
const RIVAL_RESEARCH_PRIORITY: &[Advancement] = &[
    Advancement::BronzeWorking,
    Advancement::Pottery,
    Advancement::Masonry,
    Advancement::Wheel,
    Advancement::Alphabet,
    Advancement::IronWorking,
    Advancement::Writing,
    Advancement::CodeOfLaws,
    Advancement::Currency,
    Advancement::CeremonialBurial,
    Advancement::HorsebackRiding,
    Advancement::Mysticism,
    Advancement::Mathematics,
    Advancement::MapMaking,
    Advancement::Construction,
    Advancement::Philosophy,
    Advancement::Trade,
    Advancement::Literacy,
];

/// One step a rival civilization's unit took during its turn: the unit and the
/// tiles it left and entered. Recorded per move so the TUI can replay rival
/// movements as they happened (starting tile, then ending tile, per unit)
/// instead of presenting the round's result as though matters had teleported.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RivalMotion {
    pub unit: UnitId,
    pub from: Location,
    pub to: Location,
}

impl Engine {
    /// Drain the record of every rival unit step that the last `EndTurn`
    /// performed, oldest first. The TUI replays these as an animation once the
    /// round resolves back to the human's view.
    pub fn drain_rival_motion(&mut self) -> Vec<RivalMotion> {
        std::mem::take(&mut self.motion)
    }

    /// Play a rival civilization's turn: choose a research project, keep its
    /// cities' production belts filled, and direct the settlers and military
    /// still in the field. Every unit move goes out through the ordinary
    /// `move_unit` path so the movement, reveal, transport and combat
    /// invariants hold, and each step is recorded for the TUI's replay.
    ///
    /// The AI is deliberately deterministic: it never draws from `self.rng`,
    /// so the combat RNG stream is untouched and any given turn unfolds
    /// identically from the same world.
    pub(super) fn run_rival_turn(&mut self) {
        self.rival_research();
        self.rival_production();
        self.rival_units();
    }

    fn rival_research(&mut self) {
        let ai = self.current_player_index;
        if self.game.advancement_in_progress(ai).is_some() {
            return;
        }
        if let Some(target) = self.rival_research_target(ai) {
            self.set_research_target(target);
        }
    }

    /// The next advancement for a rival to research, following the preferred
    /// ladder; when nothing there is open yet, the cheapest researchable.
    fn rival_research_target(&self, ai: PlayerId) -> Option<Advancement> {
        if let Some(target) = RIVAL_RESEARCH_PRIORITY
            .iter()
            .copied()
            .find(|adv| self.game.can_research(ai, *adv))
        {
            return Some(target);
        }
        self.game.players[ai.index()]
            .researchable_advancements()
            .into_iter()
            .min_by_key(|adv| adv.cost())
    }

    fn rival_production(&mut self) {
        let ai = self.current_player_index;
        let city_ids: Vec<CityId> = self
            .game
            .cities
            .iter()
            .filter(|city| city.owner() == ai)
            .map(|city| city.id())
            .collect();
        for city_id in city_ids {
            let occupied = self
                .game
                .cities
                .iter()
                .find(|c| c.id() == city_id)
                .is_some_and(|city| city.production_target().is_some());
            if occupied {
                continue;
            }
            if let Some(target) = self.rival_production_pick(ai, city_id) {
                self.set_production(city_id, target);
            }
        }
    }

    /// What the idle `city_id` should build: the cheapest military unit while
    /// any of the rival's cities lacks a guard, a settler until the expansion
    /// target is reached, and the cheapest unit of any kind beyond that.
    fn rival_production_pick(&self, ai: PlayerId, city_id: CityId) -> Option<ProductionTarget> {
        let choices = self.production_choices(city_id);
        let cheapest_unit = || {
            choices
                .iter()
                .filter(|target| matches!(target, ProductionTarget::Unit(_)))
                .min_by_key(|target| target.resource_cost())
                .copied()
        };
        let cheapest_military = || {
            choices
                .iter()
                .filter(|target| {
                    matches!(target, ProductionTarget::Unit(class) if Self::is_military_class(*class))
                })
                .min_by_key(|target| target.resource_cost())
                .copied()
        };
        if self.has_ungarrisoned_city(ai) {
            return cheapest_military();
        }
        let city_count = self
            .game
            .cities
            .iter()
            .filter(|city| city.owner() == ai)
            .count();
        if city_count < RIVAL_EXPANSION_TARGET_CITIES {
            if let Some(settler) = choices
                .iter()
                .copied()
                .find(|target| *target == ProductionTarget::Unit(UnitClass::Settler))
            {
                return Some(settler);
            }
            return cheapest_unit();
        }
        cheapest_unit()
    }

    /// Whether the unit is a land-bound combatant (a soldier rather than a
    /// settler, worker, diplomat or any ship).
    fn is_military_class(class: UnitClass) -> bool {
        class.attack() > 0 && !class.can_travel_water()
    }

    fn is_military(unit: &Unit) -> bool {
        Self::is_military_class(unit.unit_class)
    }

    /// Whether a friendly non-transported military unit stands on or next to
    /// the city. Adjacency is plain (no wrap): garrisons are local guards.
    fn city_is_garrisoned(&self, owner: PlayerId, city: &City) -> bool {
        self.game.units.iter().any(|unit| {
            unit.owner() == owner
                && !unit.is_transported()
                && Self::is_military(unit)
                && (unit.location == city.location
                    || (unit.location.x.abs_diff(city.location.x) <= 1
                        && unit.location.y.abs_diff(city.location.y) <= 1))
        })
    }

    fn has_ungarrisoned_city(&self, owner: PlayerId) -> bool {
        self.game
            .cities
            .iter()
            .any(|city| city.owner() == owner && !self.city_is_garrisoned(owner, city))
    }

    fn rival_units(&mut self) {
        let ai = self.current_player_index;
        // Settlers first: found in place when already on the best explored
        // land site, otherwise walk toward it.
        for unit_id in self.game.owned_units(ai) {
            let ready = self.owned_unit(unit_id).is_some_and(|unit| {
                unit.unit_class == UnitClass::Settler
                    && !unit.is_transported()
                    && unit.order() == UnitOrder::Idle
            });
            if ready {
                self.rival_settle(unit_id);
            }
        }
        self.rival_garrison(ai);
    }

    fn rival_settle(&mut self, unit_id: UnitId) {
        let ai = self.current_player_index;
        let Some(goal) = self.best_settlement_site(ai, unit_id) else {
            return;
        };
        if self
            .owned_unit(unit_id)
            .is_some_and(|unit| unit.location == goal)
        {
            let name = self.rival_city_name(ai);
            self.found_city(unit_id, name);
            return;
        }
        self.march_towards(unit_id, goal);
    }

    /// The most attractive explored land site for a new city, or `None` when
    /// nothing is reachable. Scores by resource yield, prefers food, and never
    /// founds on a tile another city occupies. Sites are picked in tiers, all
    /// measured against the union of *every* civilization's 21-tile working
    /// grid — the rival's own, its fellow rivals' and the human's — so one
    /// rival never crowds another and nobody plops a city inside a foreign
    /// city's harvest:
    ///
    /// 1. open — shares no more than `CITY_FOOTPRINT_MAX_OVERLAP` tiles with
    ///    the union (sits at least Chebyshev 4 from every city). Always wins.
    /// 2. acceptable — shares no more than `CITY_FOOTPRINT_ACCEPTABLE_OVERLAP`
    ///    tiles (at least Chebyshev 3 out).
    /// 3. frontier — the nearest explored land tile touching unexplored ground,
    ///    marched to so the settler's reveals open up new country.
    /// 4. crowded — the best-scored tile, kept only for a fully explored map
    ///    where the tiers above found nothing, so a rival still fills its
    ///    continent rather than stall.
    ///
    /// A target only ever comes from explored tiles: the rival never plots a
    /// course into unexplored ground (the frontier goal is itself explored;
    /// only the march's final step reveals into the fog). Within the tiers the
    /// pick is deterministic: best score first, then the site nearest to the
    /// settler, then the smallest tile — so a settler arriving in open country
    /// founds at the first good land it reaches rather than chase across the
    /// map.
    fn best_settlement_site(&self, ai: PlayerId, unit_id: UnitId) -> Option<Location> {
        let unit = self.owned_unit(unit_id)?;
        let map_w = self.game.map.width;
        let occupied: Vec<Location> = self.game.cities.iter().map(|city| city.location).collect();
        let all_footprints: Vec<Location> = self
            .game
            .cities
            .iter()
            .flat_map(|city| self.game.city_footprint(city.location))
            .collect();
        let mut open: Vec<(Location, i32)> = Vec::new();
        let mut acceptable: Vec<(Location, i32)> = Vec::new();
        let mut crowded: Vec<(Location, i32)> = Vec::new();
        for y in 0..self.game.map.height {
            for x in 0..self.game.map.width {
                if !self.game.players[ai.index()].explored_at(x, y) {
                    continue;
                }
                let location = Location::new(x as u16, y as u16);
                let tile = self.game.map.tile_at(location);
                if !tile.terrain.is_land() || tile.terrain == Terrain::Tundra {
                    continue;
                }
                if occupied.contains(&location) {
                    continue;
                }
                let score = tile.yields_resources() as i32 * 2 + tile.yields_food() as i32;
                let shared = self
                    .game
                    .city_footprint(location)
                    .into_iter()
                    .filter(|tile| all_footprints.contains(tile))
                    .count();
                if shared <= CITY_FOOTPRINT_MAX_OVERLAP {
                    open.push((location, score));
                } else if shared <= CITY_FOOTPRINT_ACCEPTABLE_OVERLAP {
                    acceptable.push((location, score));
                } else {
                    crowded.push((location, score));
                }
            }
        }
        // Deterministic: best score first; among equals the site nearest to the
        // settler, so an arriving settler stops at the first good land it meets
        // instead of chasing a marginal better-tile clear across the map; ties
        // then broken by the smallest tile.
        let rank = |sites: &mut Vec<(Location, i32)>| {
            sites.sort_by(|(a, score_a), (b, score_b)| {
                score_b
                    .cmp(score_a)
                    .then_with(|| {
                        Self::chebyshev(*a, unit.location, map_w).cmp(&Self::chebyshev(
                            *b,
                            unit.location,
                            map_w,
                        ))
                    })
                    .then_with(|| a.x.cmp(&b.x))
                    .then_with(|| a.y.cmp(&b.y))
            });
        };
        rank(&mut open);
        rank(&mut acceptable);
        rank(&mut crowded);
        if let Some((location, _)) = open.first() {
            return Some(*location);
        }
        if let Some((location, _)) = acceptable.first() {
            return Some(*location);
        }
        if let Some(location) = self.nearest_frontier_edge(ai, unit_id) {
            return Some(location);
        }
        crowded.first().map(|(location, _)| *location)
    }

    /// The nearest explored land tile that touches unexplored ground, if any.
    /// The early game's only field of view is the founding city's own footprint,
    /// where every visible tile counts as crowded: rather than found in its own
    /// lap, the settler marches to the edge and lets its reveals open up new
    /// country. `None` when the map is fully explored or no such edge exists.
    /// Deterministic: nearest to the unit by wrapped Chebyshev distance, ties
    /// broken by the first tile in scan order (smallest `y`, then `x`).
    fn nearest_frontier_edge(&self, ai: PlayerId, unit_id: UnitId) -> Option<Location> {
        let unit = self.owned_unit(unit_id)?;
        let map_w = self.game.map.width;
        let player = &self.game.players[ai.index()];
        (0..self.game.map.height)
            .flat_map(|y| (0..self.game.map.width).map(move |x| Location::new(x as u16, y as u16)))
            .filter(|location| player.explored_at(location.x as usize, location.y as usize))
            .filter(|location| {
                let tile = self.game.map.tile_at(*location);
                tile.terrain.is_land() && tile.terrain != Terrain::Tundra
            })
            .filter(|location| {
                Direction::iter().any(|direction| {
                    self.game
                        .map
                        .destination(*location, direction)
                        .is_some_and(|neighbour| {
                            !player.explored_at(neighbour.x as usize, neighbour.y as usize)
                        })
                })
            })
            .min_by_key(|location| Self::chebyshev(unit.location, *location, map_w))
    }

    /// The name for a rival's next city: its civilization's names in order of
    /// founding, or a numbered fallback once those run out.
    fn rival_city_name(&self, ai: PlayerId) -> String {
        let count = self
            .game
            .cities
            .iter()
            .filter(|city| city.owner() == ai)
            .count();
        self.game.players[ai.index()]
            .civilization
            .city_names()
            .get(count)
            .map(|name| (*name).to_string())
            .unwrap_or_else(|| format!("City {}", count + 1))
    }

    fn rival_garrison(&mut self, ai: PlayerId) {
        let mut ungarrisoned: Vec<CityId> = self
            .game
            .cities
            .iter()
            .filter(|city| city.owner() == ai && !self.city_is_garrisoned(ai, city))
            .map(|city| city.id())
            .collect();
        for unit_id in self.game.owned_units(ai) {
            if ungarrisoned.is_empty() {
                break;
            }
            let ready = self.owned_unit(unit_id).is_some_and(|unit| {
                Self::is_military(unit) && !unit.is_transported() && unit.order() == UnitOrder::Idle
            });
            if !ready {
                continue;
            }
            let Some((city_id, goal)) = self.nearest_ungarrisoned(unit_id, &ungarrisoned) else {
                continue;
            };
            let stationed = self.owned_unit(unit_id).is_some_and(|unit| {
                let city = self
                    .game
                    .cities
                    .iter()
                    .find(|c| c.id() == city_id)
                    .expect("ungarrisoned city still exists");
                unit.location == city.location
                    || (unit.location.x.abs_diff(city.location.x) <= 1
                        && unit.location.y.abs_diff(city.location.y) <= 1)
            });
            if stationed {
                self.fortify(unit_id);
                ungarrisoned.retain(|id| *id != city_id);
            } else {
                self.march_towards(unit_id, goal);
            }
        }
    }

    /// The ungarrisoned city closest to `unit_id`. `ungarrisoned` is the set
    /// of cities still awaiting a guard.
    fn nearest_ungarrisoned(
        &self,
        unit_id: UnitId,
        ungarrisoned: &[CityId],
    ) -> Option<(CityId, Location)> {
        let unit = self.owned_unit(unit_id)?;
        let map_w = self.game.map.width;
        ungarrisoned
            .iter()
            .copied()
            .filter_map(|city_id| {
                let city = self.game.cities.iter().find(|c| c.id() == city_id)?;
                Some((city_id, city.location))
            })
            .min_by_key(|(_, location)| Self::chebyshev(unit.location, *location, map_w))
    }

    /// How far the unit should travel in a single turn: keep stepping toward
    /// `goal` while moves remain and each step closes the distance.
    fn march_towards(&mut self, unit_id: UnitId, goal: Location) {
        while self
            .owned_unit(unit_id)
            .is_some_and(|unit| unit.moves_remaining() > 0 && unit.location != goal)
        {
            let Some(direction) = self.step_towards(unit_id, goal) else {
                break;
            };
            self.move_unit(unit_id, direction);
        }
    }

    /// The best one-of-eight step from the unit's tile toward `goal`: any
    /// direction the unit may actually enter that moves it closer, choosing
    /// the closest by wrapped Chebyshev distance with `Direction::iter` order
    /// as the deterministic tie-break. `None` when no step gets closer.
    fn step_towards(&self, unit_id: UnitId, goal: Location) -> Option<Direction> {
        let unit = self.owned_unit(unit_id)?;
        let map_w = self.game.map.width;
        Direction::iter()
            .filter(|direction| {
                self.ensure_can_move(unit_id, *direction).is_ok()
                    && self
                        .game
                        .map
                        .destination(unit.location, *direction)
                        .is_some_and(|destination| {
                            Self::chebyshev(destination, goal, map_w)
                                < Self::chebyshev(unit.location, goal, map_w)
                        })
            })
            .min_by_key(|direction| {
                let destination = self
                    .game
                    .map
                    .destination(unit.location, *direction)
                    .expect("closer destination exists");
                Self::chebyshev(destination, goal, map_w)
            })
    }

    /// Chebyshev distance between two world tiles, wrapping east/west.
    fn chebyshev(a: Location, b: Location, map_w: usize) -> usize {
        let dx = a.x.abs_diff(b.x) as usize;
        let dy = a.y.abs_diff(b.y) as usize;
        dx.min(map_w - dx).max(dy)
    }
}
