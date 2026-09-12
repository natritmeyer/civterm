use crate::game_engine::{Command, Event, Player};
use crate::model::advancements::Advancement;
use crate::model::cartography::Location;
use crate::model::cartography::generation::MapGenerator;
use crate::model::cities::CityId;
use crate::model::civilizations::{Civilization, PlayerId};
use crate::model::geography::Terrain;
use crate::model::units::UnitClass;
#[cfg(test)]
use crate::model::units::UnitId;

use crate::game_engine::game::Game;
use crate::utils::Rng;
use strum::IntoEnumIterator;

const DEFAULT_SEED: u64 = 0xC0FFEE;

/// Default width of a generated world, mirroring classic Civ: 80 × 50.
pub const DEFAULT_MAP_WIDTH: usize = 80;
/// Default height of a generated world.
pub const DEFAULT_MAP_HEIGHT: usize = 50;

pub struct Engine {
    pub(crate) game: Game,
    pub(crate) turn: u32,
    pub(crate) current_player_index: PlayerId,
    pub(crate) events: Vec<Event>,
    pub(crate) rng: Rng,
}

impl Default for Engine {
    fn default() -> Self {
        let first = Player::new(Civilization::English);
        Engine::with_seed(
            DEFAULT_MAP_WIDTH,
            DEFAULT_MAP_HEIGHT,
            first,
            Vec::new(),
            DEFAULT_SEED,
        )
    }
}

impl Engine {
    pub fn new(width: usize, height: usize, first: Player, rest: Vec<Player>) -> Self {
        Engine::with_seed(width, height, first, rest, DEFAULT_SEED)
    }

    /// Build an engine whose RNG is seeded from the system clock, so a new
    /// game draws a fresh random world (map + starting positions).
    pub fn new_random(width: usize, height: usize, first: Player, rest: Vec<Player>) -> Self {
        Engine::with_seed(width, height, first, rest, crate::utils::random_seed())
    }

    pub fn with_seed(
        width: usize,
        height: usize,
        first: Player,
        rest: Vec<Player>,
        seed: u64,
    ) -> Self {
        Engine {
            game: Game::new(width, height, first, rest),
            turn: 1,
            current_player_index: PlayerId::new(0),
            events: Vec::new(),
            rng: Rng::new(seed),
        }
    }

    /// Lay down real terrain and give every player a starting settler on a
    /// random land tile. The settler's tile and its surroundings are revealed
    /// to that player. Distinct starting tiles are chosen for each player.
    pub fn populate_starting_world(&mut self) {
        let width = self.game.map.width;
        let height = self.game.map.height;

        let mut generator = MapGenerator::with_rng(self.rng.clone());
        let map = generator.generate(width, height);
        self.game.map = map;

        // Collect land tiles once so each player lands on a different one.
        // Tundra is excluded: no civilization starts in the tundra wastes.
        let mut land: Vec<Location> = (0..height)
            .flat_map(|y| (0..width).map(move |x| Location::new(x as u16, y as u16)))
            .filter(|location| {
                let terrain = self.game.map.tile_at(*location).terrain;
                terrain.is_land() && terrain != Terrain::Tundra
            })
            .collect();

        let player_count = self.game.players.len();
        for index in 0..player_count {
            if land.is_empty() {
                break;
            }
            let pick = self.rng.in_range(land.len() as u32) as usize;
            let location = land.swap_remove(pick);
            let owner = PlayerId::new(index);
            self.game
                .spawn_unit(UnitClass::Settler, location, owner, CityId::new(0));
        }
    }

    pub fn submit(&mut self, command: Command) -> Vec<Event> {
        match command {
            Command::Move { unit, direction } => self.move_unit(unit, direction),
            Command::Fortify { unit } => self.fortify(unit),
            Command::Sentry { unit } => self.sentry(unit),
            Command::Work { unit, improvement } => self.work(unit, improvement),
            Command::CancelOrder { unit } => self.cancel_order(unit),
            Command::FoundCity { unit, name } => self.found_city(unit, name),
            Command::SetProductionTarget { city, target } => self.set_production(city, target),
            Command::DeclareWar { opponent } => self.declare_war(opponent),
            Command::MakePeace { opponent } => self.make_peace(opponent),
            Command::SetResearchTarget { advancement } => self.set_research_target(advancement),
            Command::EndTurn => self.end_turn(),
        }
        std::mem::take(&mut self.events)
    }

    /// Place a fresh unit for `owner`. Exposed only to the crate's tests so
    /// `src/tui` can stage combat scenarios; the game itself never calls it.
    #[cfg(test)]
    pub(crate) fn spawn_unit(
        &mut self,
        unit_class: UnitClass,
        location: Location,
        owner: PlayerId,
        home_city: CityId,
    ) -> UnitId {
        self.game.spawn_unit(unit_class, location, owner, home_city)
    }

    /// The player governed by `civilization`, if that civilization is in play.
    pub fn player_id_of(&self, civilization: Civilization) -> Option<PlayerId> {
        self.game
            .players
            .iter()
            .position(|player| player.civilization == civilization)
            .map(PlayerId::new)
    }

    /// The advancements `player` has already discovered, in discovery order.
    pub fn player_advances(&self, player: PlayerId) -> Vec<Advancement> {
        self.game.players[player.index()].advances_made().to_vec()
    }

    /// The advancements `player` may begin researching right now: not yet
    /// discovered, with every prerequisite already discovered.
    pub fn researchable_advancements_for(&self, player: PlayerId) -> Vec<Advancement> {
        Advancement::iter()
            .filter(|advancement| self.game.can_research(player, *advancement))
            .collect()
    }
}
