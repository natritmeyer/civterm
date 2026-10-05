use super::*;

const HIT_POINTS: u32 = 10;

/// Defense multiplier a unit gains for standing on its own city tile when
/// that city has built CityWalls. It applies to any defender there, fortified
/// or not, stacked on top of the ordinary home-city bonus.
const CITY_WALLS_DEFENSE_BONUS: u32 = 2;

/// Defense multiplier a unit gains while fortified, the `f` order. It applies
/// wherever the unit is standing and stacks with the terrain, home-city and
/// walls bonuses, so a fortified garrison behind walls on a hill is the
/// hardest tile on the map to take.
const FORTIFIED_DEFENSE_BONUS: u32 = 3;

use crate::game_engine::Event;
use crate::model::cartography::Location;
use crate::model::cities::{CityId, CityImprovement};
use crate::model::civilizations::PlayerId;
use crate::model::geography::Terrain;
use crate::model::units::UnitOrder;
use crate::model::units::{Unit, UnitId};

impl Engine {
    /// A siege engine takes a city apart rather than beating its garrison. It
    /// spends its whole turn on one improvement, preferring the walls: tearing
    /// them down strips the `CITY_WALLS_DEFENSE_BONUS` from every defender in
    /// the city, which is the point of the exercise. Once the walls are flat
    /// an ordinary warrior can take the tile as it does any other.
    ///
    /// The bombardment never captures the city and never touches a unit, so a
    /// catapult sitting in a besieged capital is a very thin place to be.
    pub(super) fn bombard_city(&mut self, attacker_idx: usize, tile: Location) {
        let attacker_id = self.game.units[attacker_idx].id();
        let attacker_owner = self.game.units[attacker_idx].owner();
        let city_idx = self
            .game
            .cities
            .iter()
            .position(|c| c.location == tile && self.game.at_war(attacker_owner, c.owner()));
        let Some(city_idx) = city_idx else {
            self.game.units[attacker_idx].spend_turn();
            return;
        };
        // Walls first: they are the improvement worth removing, and the only
        // one that is a defence in its own right.
        let target = self.game.cities[city_idx]
            .improvements()
            .iter()
            .find(|i| **i == CityImprovement::CityWalls)
            .or_else(|| self.game.cities[city_idx].improvements().first())
            .copied();
        let city_name = self.game.cities[city_idx].name.clone();
        match target {
            Some(improvement) => {
                self.game.cities[city_idx].remove_improvement(improvement);
                self.events.push(Event::between(
                    attacker_owner,
                    self.game.cities[city_idx].owner(),
                    format!(
                        "Unit {} bombards the {} of {}",
                        attacker_id.index(),
                        improvement.name(),
                        city_name
                    ),
                ));
            }
            None => self.events.push(Event::between(
                attacker_owner,
                self.game.cities[city_idx].owner(),
                format!(
                    "Unit {} bombards {} but finds nothing left to break",
                    attacker_id.index(),
                    city_name
                ),
            )),
        }
        self.game.units[attacker_idx].spend_turn();
    }
    pub(super) fn resolve_move_combat(&mut self, attacker: UnitId, tile: Location) {
        let attacker_idx = self
            .game
            .units
            .iter()
            .position(|u| u.id() == attacker)
            .expect("the moving unit exists");
        // A siege engine has no quarrel with the garrison: it bombards the city.
        if self.game.units[attacker_idx].unit_class.sieges() {
            self.bombard_city(attacker_idx, tile);
            return;
        }
        let defender_idx = self.select_defender(attacker_idx, tile);

        let attacker_id = self.game.units[attacker_idx].id();
        let defender_id = self.game.units[defender_idx].id();
        let attacker_owner = self.game.units[attacker_idx].owner();
        let defender_owner = self.game.units[defender_idx].owner();
        self.events.push(Event::between(
            attacker_owner,
            defender_owner,
            format!(
                "Unit {} attacks Unit {}",
                attacker_id.index(),
                defender_id.index()
            ),
        ));

        let attacker_power = self.attacker_power(&self.game.units[attacker_idx]);
        let defender_power = self.defender_power(&self.game.units[defender_idx]);
        let attacker_won = self.resolve_combat(attacker_power, defender_power);

        if attacker_won {
            let was_veteran = self.game.units[attacker_idx].is_veteran();
            self.game.remove_unit(defender_id);
            let lost_cargo = self.game.disband_cargo_of(defender_id);
            if lost_cargo > 0 {
                self.events.push(Event::for_player(
                    defender_owner,
                    format!(
                        "{lost_cargo} transported units are lost with Unit {}",
                        defender_id.index()
                    ),
                ));
            }
            let tile_is_clear = !self.game.units.iter().any(|unit| {
                !unit.is_transported()
                    && unit.location == tile
                    && self.game.at_war(attacker_owner, unit.owner())
            });
            // Advancing onto the cleared tile reveals the ring around it.
            if tile_is_clear {
                self.game.reveal_tiles_at(attacker_owner, tile);
            }
            let attacker_unit = self.owned_unit_mut(attacker_id).unwrap();
            if tile_is_clear {
                attacker_unit.location = tile;
            }
            // A transported attacker stepped off its ship to fight; cargo
            // still aboard the winner follows it onto a cleared tile.
            attacker_unit.disembark();
            attacker_unit.spend_turn();
            if !was_veteran {
                attacker_unit.promote();
            }
            self.game.sync_cargo(attacker_id);
            self.events.push(Event::between(
                attacker_owner,
                defender_owner,
                format!(
                    "Unit {} defeats Unit {}",
                    attacker_id.index(),
                    defender_id.index()
                ),
            ));
            // Winning the fight on a city tile conquers the now-undefended
            // city for the attacker: the conqueror stands inside a city that
            // has just fallen to them, so it flips colour the instant the
            // defender falls rather than a round later.
            if tile_is_clear
                && let Some(city) = self.game.cities.iter().find(|c| {
                    c.location == tile
                        && c.owner() != attacker_owner
                        && self.game.at_war(attacker_owner, c.owner())
                })
            {
                self.take_captured_city(city.id());
            }
            self.eliminate_if_annihilated(defender_owner);
        } else {
            self.game.remove_unit(attacker_id);
            let lost_cargo = self.game.disband_cargo_of(attacker_id);
            if lost_cargo > 0 {
                self.events.push(Event::for_player(
                    attacker_owner,
                    format!(
                        "{lost_cargo} transported units are lost with Unit {}",
                        attacker_id.index()
                    ),
                ));
            }
            self.events.push(Event::between(
                attacker_owner,
                defender_owner,
                format!(
                    "Unit {} repels Unit {}",
                    defender_id.index(),
                    attacker_id.index()
                ),
            ));
            self.eliminate_if_annihilated(attacker_owner);
        }
    }
    /// Deliver a conquered city to the current player after its guard has
    /// fallen. A city of population one is doomed: it is overrun and destroyed
    /// outright, removed from the game entirely — an empty ruin holds nothing
    /// for the conqueror. Any larger city transfers ownership, flipping its
    /// tile to the conqueror's colour then and there. Units homed to the
    /// fallen city are disbanded under either fate, and a loser left without
    /// a single city is eliminated. The conquering unit is already on the
    /// city tile; the caller handles its placement and movement.
    pub(super) fn take_captured_city(&mut self, city_id: CityId) {
        let city_name = self
            .game
            .cities
            .iter()
            .find(|c| c.id() == city_id)
            .expect("the conquered city exists")
            .name
            .clone();
        let old_owner = self
            .game
            .cities
            .iter()
            .find(|c| c.id() == city_id)
            .unwrap()
            .owner();
        let overrun = self
            .game
            .cities
            .iter()
            .find(|c| c.id() == city_id)
            .unwrap()
            .population()
            == 1;
        let disbanded = self.game.disband_units_homed_to(city_id);
        let conqueror = self.game.players[self.current_player_index.index()].civilization;
        let loser = self.game.players[old_owner.index()].civilization;
        if overrun {
            self.game.cities.retain(|c| c.id() != city_id);
            self.events.push(Event::between(
                self.current_player_index,
                old_owner,
                format!("{conqueror:?} destroy {city_name} (formerly {loser:?}'s)"),
            ));
        } else {
            self.game
                .cities
                .iter_mut()
                .find(|c| c.id() == city_id)
                .unwrap()
                .change_owner(self.current_player_index);
            self.events.push(Event::between(
                self.current_player_index,
                old_owner,
                format!("{conqueror:?} capture {city_name} (formerly {loser:?}'s)"),
            ));
        }
        if disbanded > 0 {
            self.events.push(Event::between(
                self.current_player_index,
                old_owner,
                format!("{disbanded} units disband with the loss of {city_name}"),
            ));
        }
        self.eliminate_if_cityless(old_owner);
    }
    /// The attacker has advanced onto an undefended foreign city tile and
    /// takes it for the current player — ownership transfers, or a
    /// population-one city falls and is destroyed. Units homed to the
    /// captured city are disbanded. The attacking unit advances onto the
    /// city tile.
    pub(super) fn capture_city(&mut self, city_id: CityId, unit: UnitId, destination: Location) {
        self.take_captured_city(city_id);
        let mut_unit = self.owned_unit_mut(unit).unwrap();
        mut_unit.location = destination;
        mut_unit.disembark();
        mut_unit.spend_turn();
        self.game
            .reveal_tiles_at(self.current_player_index, destination);
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
        self.events.push(Event::for_player(
            player,
            format!("{civilization:?} has been eliminated"),
        ));
        if disbanded > 0 {
            self.events.push(Event::for_player(
                player,
                format!("{disbanded} units disband with the loss of {civilization:?}"),
            ));
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
                    && !unit.is_transported()
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
        let city_is_walled = self.game.cities.iter().any(|city| {
            city.location == unit.location
                && city.owner() == unit.owner()
                && city.improvements().contains(&CityImprovement::CityWalls)
        });
        if city_is_walled {
            power *= CITY_WALLS_DEFENSE_BONUS;
        }
        if unit.is_veteran() {
            power = power * 3 / 2;
        }
        if unit.order() == UnitOrder::Fortified {
            power = power * FORTIFIED_DEFENSE_BONUS / 2;
        }
        power
    }
    /// The attacker's real chance of winning the fight, as a percentage.
    ///
    /// Combat is a race to `HIT_POINTS`, not a single roll: each side lands one
    /// hit per round with probability proportional to its power, so the chance
    /// of winning is `a^n / (a^n + d^n)`, not `a / (a + d)`. The two agree at
    /// even odds but diverge violently past it — a 60-90 fight is two thirds of
    /// the attacker's power and yet wins barely one time in fifty. Reading the
    /// linear share would have the rival throwing warriors at fortified walls
    /// that will eat them every time.
    pub(super) fn win_chance_percent(attacker: u32, defender: u32) -> u32 {
        // Both sides carry `HIT_POINTS` health, and a power of `x` wins a round
        // with probability `x / (a + d)`. u128 keeps `a^10` exact: powers here
        // run to a few hundred at most, and even 10_000^10 fits comfortably.
        const H: u32 = HIT_POINTS;
        let (a, d) = (attacker as u128, defender as u128);
        let (a_hits, d_hits) = (a.pow(H), d.pow(H));
        let total = a_hits + d_hits;
        if total == 0 {
            // Two powerless units cannot trade a hit, so neither side can lose.
            return 50;
        }
        (a_hits * 100 / total) as u32
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
