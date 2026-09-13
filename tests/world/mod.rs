mod args;
mod steps;

use std::fmt;
use std::ops::{Deref, DerefMut};

use civterm::game_engine::{
    Command, DEFAULT_MAP_HEIGHT, DEFAULT_MAP_WIDTH, Engine, GameView, Player,
};
use civterm::model::cartography::{Direction, Location};
use civterm::model::cities::CityId;
use civterm::model::civilizations::{Civilization, PlayerId};
use civterm::model::units::{Unit, UnitId};

use crate::world::args::{StartingCondition, translate};

/// A wrapper around the engine that a game world requires `Debug` for; the
/// engine itself carries none (its RNG does not implement it).
pub struct Game(Engine);

impl fmt::Debug for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Game(Engine)")
    }
}

impl Deref for Game {
    type Target = Engine;

    fn deref(&self) -> &Engine {
        &self.0
    }
}

impl DerefMut for Game {
    fn deref_mut(&mut self) -> &mut Engine {
        &mut self.0
    }
}

impl Game {
    /// Build a seeded game where `first` moves first and `rest` follow.
    fn new(seed: u64, first: Civilization, rest: &[Civilization]) -> Self {
        let mut engine = Engine::with_seed(
            DEFAULT_MAP_WIDTH,
            DEFAULT_MAP_HEIGHT,
            Player::new(first),
            rest.iter().map(|civ| Player::new(*civ)).collect(),
            seed,
        );
        engine.populate_starting_world();
        Game(engine)
    }
}

/// The state threaded through a scenario's steps.
#[derive(Debug, Default, cucumber::World)]
pub struct CivWorld {
    game: Option<Game>,
    english: Option<PlayerId>,
    zulu: Option<PlayerId>,
    roman: Option<PlayerId>,
    settler: Option<UnitId>,
    city: Option<CityId>,
    origin: Option<Location>,
    last_events: Vec<String>,
}

impl CivWorld {
    /// Start a solo game on the seed whose world satisfies `condition`.
    pub fn settle(&mut self, civ: Civilization, condition: StartingCondition) {
        self.start(Game::new(condition.seed(), civ, &[]), civ);
    }

    /// Start a three-civilization game. The first civilization moves first.
    pub fn start_trio(&mut self, first: Civilization, second: Civilization, third: Civilization) {
        let game = Game::new(1, first, &[second, third]);
        let second_player = game.player_id_of(second).unwrap();
        let third_player = game.player_id_of(third).unwrap();
        self.start(game, first);
        self.record_player(second, second_player);
        self.record_player(third, third_player);
    }

    /// Adopt a freshly built game as this world's state.
    fn start(&mut self, game: Game, first: Civilization) {
        let player = game.current_player_id();
        self.record_player(first, player);
        let (settler, origin) = current_settler(&game);
        self.game = Some(game);
        self.settler = Some(settler);
        self.origin = Some(origin);
        self.last_events = Vec::new();
    }

    /// File away the player id governing `civ`, for later steps.
    fn record_player(&mut self, civ: Civilization, player: PlayerId) {
        match civ {
            Civilization::English => self.english = Some(player),
            Civilization::Zulu => self.zulu = Some(player),
            Civilization::Roman => self.roman = Some(player),
            _ => unreachable!("features only speak about the English, Zulu and Roman"),
        }
    }

    /// The id of the player governing `civ` in the current game.
    pub fn player_id(&self, civ: Civilization) -> PlayerId {
        match civ {
            Civilization::English => self.english,
            Civilization::Zulu => self.zulu,
            Civilization::Roman => self.roman,
            _ => None,
        }
        .expect("the civilization has been recorded in this game")
    }

    pub fn engine(&self) -> &Engine {
        &self.game.as_ref().expect("a game has been started").0
    }

    fn engine_mut(&mut self) -> &mut Engine {
        &mut self.game.as_mut().expect("a game has been started").0
    }

    /// The recorded starting settler's id.
    pub fn settler_id(&self) -> UnitId {
        self.settler.expect("a settler has been recorded")
    }

    /// The recorded starting settler, still alive at its current position.
    pub fn settler(&self) -> &Unit {
        let id = self.settler_id();
        self.engine()
            .player_units()
            .into_iter()
            .find(|unit| unit.id() == id)
            .expect("the recorded settler still exists")
    }

    /// Where the settler stands right now.
    pub fn settler_location(&self) -> Location {
        self.settler().location
    }

    /// The tile the settler started on.
    pub fn origin(&self) -> Location {
        self.origin.expect("a settler has been recorded")
    }

    /// Remember the city the current player founded.
    pub fn record_city(&mut self) {
        let id = self.engine().player_cities()[0].id();
        self.city = Some(id);
    }

    /// The city founded in the current game.
    pub fn city(&self) -> CityId {
        self.city.expect("a city has been founded")
    }

    /// The civilization governing `id`'s city.
    pub fn city_owner(&self, id: CityId) -> Civilization {
        let owner = self
            .engine()
            .city(id)
            .expect("the city still exists")
            .owner();
        self.engine().civilization_of(owner)
    }

    /// Run one engine command and remember the events it emitted.
    pub fn submit(&mut self, command: Command) {
        self.last_events = self
            .engine_mut()
            .submit(command)
            .into_iter()
            .map(|event| event.message().to_string())
            .collect();
    }

    /// End `count` full turns, one step at a time.
    pub fn end_turns(&mut self, count: u32) {
        for _ in 0..count {
            self.submit(Command::EndTurn);
        }
    }

    /// Whether the last command's events include exactly `message`.
    pub fn has_event(&self, message: &str) -> bool {
        self.last_events.iter().any(|event| event == message)
    }

    /// Whether the last command's events include one containing `part`.
    pub fn has_event_containing(&self, part: &str) -> bool {
        self.last_events.iter().any(|event| event.contains(part))
    }

    /// `origin` shifted one tile toward `direction`, wrapping east/west.
    pub fn neighbour(&self, direction: Direction) -> Location {
        let engine = self.engine();
        translate(engine.width(), engine.height(), self.origin(), direction, 1)
            .expect("the destination stays on the map")
    }

    /// `origin` shifted two tiles toward `direction`, wrapping east/west.
    pub fn two_tiles_towards(&self, direction: Direction) -> Location {
        let engine = self.engine();
        translate(engine.width(), engine.height(), self.origin(), direction, 2)
            .expect("the destination stays on the map")
    }
}

/// The current player's starting settler and where it stands.
fn current_settler(game: &Game) -> (UnitId, Location) {
    let unit = game
        .player_units()
        .into_iter()
        .find(|unit| unit.unit_class == civterm::model::units::UnitClass::Settler)
        .expect("the current player has a settler");
    (unit.id(), unit.location)
}
