use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::game_engine::Engine;
use crate::game_engine::event::Event;
use crate::game_engine::game::Game;
#[cfg(test)]
use crate::game_engine::{DiplomatAudience, RivalWar};
use crate::model::civilizations::PlayerId;
use crate::model::competition::Competition;
use crate::model::difficulty::Difficulty;
#[cfg(test)]
use crate::model::units::UnitId;
use crate::utils::Rng;

/// The version of the save format the loader writes. Bump it whenever the
/// shape of `SaveData` changes destructively, and list the previous version
/// here so older files keep loading.
pub const SAVE_FORMAT_VERSION: u32 = 1;

/// Everything needed to recreate an in-flight game: the whole engine plus the
/// setup metadata (competition, difficulty) that lives on the App rather than
/// on the engine. Serialized as JSON to a `.civ` file. New fields must be
/// optional (`#[serde(default)]`) so files written by older builds still load.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveData {
    version: u32,
    turn: u32,
    current_player_index: PlayerId,
    #[serde(default)]
    events: Vec<Event>,
    rng: Rng,
    game: Game,
    competition: Competition,
    difficulty: Difficulty,
}

impl SaveData {
    /// Snapshot the engine and its companion setup metadata for writing.
    pub fn capture(engine: &Engine, competition: Competition, difficulty: Difficulty) -> Self {
        SaveData {
            version: SAVE_FORMAT_VERSION,
            turn: engine.turn,
            current_player_index: engine.current_player_index,
            events: engine.events.clone(),
            rng: engine.rng.clone(),
            game: engine.game.clone(),
            competition,
            difficulty,
        }
    }

    /// Rebuild a game from a snapshot, refusing a format newer than this build
    /// understands rather than risk silently dropping state.
    pub fn into_loaded(self) -> Result<LoadedGame, SaveError> {
        if self.version > SAVE_FORMAT_VERSION {
            return Err(SaveError::Version(self.version));
        }
        Ok(LoadedGame {
            engine: Engine {
                game: self.game,
                turn: self.turn,
                current_player_index: self.current_player_index,
                events: self.events,
                rng: self.rng,
                motion: Vec::new(),
                rival_wars: Vec::new(),
                builds: Vec::new(),
                starvations: Vec::new(),
                audiences: Vec::new(),
                investigation: None,
                steal: None,
                sabotage: None,
            },
            competition: self.competition,
            difficulty: self.difficulty,
        })
    }
}

/// A game restored from a save file: the engine plus the setup metadata the
/// app normally keeps separate from it.
#[derive(Debug)]
pub struct LoadedGame {
    pub engine: Engine,
    pub competition: Competition,
    pub difficulty: Difficulty,
}

/// Why a save or load failed, in player-facing terms.
#[derive(Debug)]
pub enum SaveError {
    Io(io::Error),
    Json(serde_json::Error),
    Version(u32),
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveError::Io(err) => write!(f, "{err}"),
            SaveError::Json(err) => write!(f, "the save file is not valid: {err}"),
            SaveError::Version(version) => write!(
                f,
                "save file version {version} is newer than this build supports ({SAVE_FORMAT_VERSION})"
            ),
        }
    }
}

impl std::error::Error for SaveError {}

/// Write `engine` and its setup metadata to `path` as readable JSON,
/// overwriting any file already there. The path is passed to `std::fs`
/// exactly as given.
pub fn save_game(
    path: impl AsRef<Path>,
    engine: &Engine,
    competition: Competition,
    difficulty: Difficulty,
) -> Result<(), SaveError> {
    let json = serde_json::to_string_pretty(&SaveData::capture(engine, competition, difficulty))
        .map_err(SaveError::Json)?;
    fs::write(path, json).map_err(SaveError::Io)
}

/// Load a saved game from `path`. The path is passed to `std::fs` exactly as
/// given.
pub fn load_game(path: impl AsRef<Path>) -> Result<LoadedGame, SaveError> {
    let text = fs::read_to_string(path).map_err(SaveError::Io)?;
    let data: SaveData = serde_json::from_str(&text).map_err(SaveError::Json)?;
    data.into_loaded()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_engine::command::Command;
    use crate::game_engine::{BuildComplete, Event, GameView, Player};
    use crate::model::cartography::{Direction, Location};
    use crate::model::cities::{CityId, ProductionTarget};
    use crate::model::civilizations::{Civilization, PlayerId};
    use crate::model::geography::Terrain;
    use crate::model::units::UnitClass;

    fn engine() -> Engine {
        let mut engine = Engine::with_seed(
            60,
            40,
            Player::new(Civilization::English),
            vec![Player::new(Civilization::Zulu)],
            42,
        );
        engine.populate_starting_world();
        engine
    }

    #[test]
    fn a_captured_game_round_trips_through_json() {
        let mut engine = engine();
        let human = PlayerId::new(0);
        // Give the snapshot real state: a moved unit, a city with production,
        // research in progress, and a diplomatic relationship.
        let settler = engine.player_units()[0].id();
        engine.submit(Command::Move {
            unit: settler,
            direction: Direction::E,
        });
        let london = engine.game.add_city(human, "London", Location::new(2, 2));
        engine.game.cities[london.index()]
            .set_production(ProductionTarget::Unit(UnitClass::Militia));
        engine.game.begin_research(human);
        engine.game.make_peace(human, PlayerId::new(1));

        let data = SaveData::capture(&engine, Competition::new(2), Difficulty::Hard);
        let json = serde_json::to_string(&data).unwrap();
        let restored: SaveData = serde_json::from_str(&json).unwrap();
        let loaded = restored.into_loaded().unwrap();

        assert_eq!(loaded.engine.game, engine.game);
        assert_eq!(loaded.engine.turn, engine.turn);
        assert_eq!(
            loaded.engine.current_player_index,
            engine.current_player_index
        );
        assert_eq!(loaded.competition, Competition::new(2));
        assert_eq!(loaded.difficulty, Difficulty::Hard);

        // The RNG restores to the same point in its stream, so the world keeps
        // drawing the same numbers after the load.
        let mut original_rng = engine.rng.clone();
        let mut loaded_rng = loaded.engine.rng.clone();
        for _ in 0..50 {
            assert_eq!(original_rng.in_range(1000), loaded_rng.in_range(1000));
        }
    }

    #[test]
    fn a_homeless_starting_settler_survives_save_and_load() {
        let engine = engine();
        // A starting settler is homed to no city. That absent reference is the
        // whole point: a placeholder id would be indistinguishable from a real
        // city once the first one is founded, so the loaded unit must still
        // read as homeless, not as homed to whatever took that id.
        let settler = engine.player_units()[0].id();
        assert_eq!(
            engine
                .game
                .units
                .iter()
                .find(|u| u.id() == settler)
                .unwrap()
                .home_city(),
            None
        );

        let data = SaveData::capture(&engine, Competition::new(2), Difficulty::Hard);
        let json = serde_json::to_string(&data).unwrap();
        let loaded = serde_json::from_str::<SaveData>(&json)
            .unwrap()
            .into_loaded()
            .unwrap();

        assert_eq!(
            loaded
                .engine
                .game
                .units
                .iter()
                .find(|u| u.id() == settler)
                .unwrap()
                .home_city(),
            None,
            "a homeless settler loads homeless"
        );
        assert_eq!(loaded.engine.game, engine.game);
    }

    #[test]
    fn a_save_written_before_home_cities_were_optional_still_loads() {
        // Older builds stored a bare city id for every unit. Such a file must
        // keep loading: serde reads the number as `Some(id)`, and `#[serde(
        // default)]` covers a unit whose field is missing altogether.
        let engine = engine();
        let settler = engine.player_units()[0].id();
        let data = SaveData::capture(&engine, Competition::new(2), Difficulty::Hard);
        let json = serde_json::to_string(&data)
            .unwrap()
            .replace("\"home_city\":null", "\"home_city\":0");

        let loaded = serde_json::from_str::<SaveData>(&json)
            .unwrap()
            .into_loaded()
            .unwrap();

        assert_eq!(
            loaded
                .engine
                .game
                .units
                .iter()
                .find(|u| u.id() == settler)
                .unwrap()
                .home_city(),
            Some(CityId::new(0)),
            "the old bare id reads as a real home city"
        );
    }

    #[test]
    fn save_game_and_load_game_write_and_read_a_file() {
        let engine = engine();
        let path = std::env::temp_dir().join(format!("civterm-save-{}.civ", std::process::id()));
        save_game(&path, &engine, Competition::new(3), Difficulty::Easy).unwrap();
        let loaded = load_game(&path).unwrap();
        assert_eq!(loaded.engine.game, engine.game);
        assert_eq!(loaded.engine.turn, engine.turn);
        assert_eq!(loaded.competition, Competition::new(3));
        assert_eq!(loaded.difficulty, Difficulty::Easy);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn loading_a_missing_file_reports_an_io_error() {
        let path = std::env::temp_dir().join("civterm-no-such-file.civ");
        assert!(matches!(load_game(&path), Err(SaveError::Io(_))));
    }

    #[test]
    fn loading_corrupt_json_reports_a_parse_error() {
        let path = std::env::temp_dir().join(format!("civterm-bad-{}.civ", std::process::id()));
        fs::write(&path, "{\"version\": ").unwrap();
        assert!(matches!(load_game(&path), Err(SaveError::Json(_))));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn a_transported_unit_survives_save_and_load() {
        // A legion ferried aboard a trireme must come back aboard the same
        // ship, standing on the ship's tile and still invisible to the map,
        // after a save/load round trip.
        let mut engine = Engine::new(4, 3, Player::new(Civilization::English), Vec::new());
        engine.game.map.tile_at_mut(Location::new(0, 1)).terrain = Terrain::Grassland;
        let trireme = engine.game.spawn_unit(
            UnitClass::Trireme,
            Location::new(1, 1),
            PlayerId::new(0),
            Some(CityId::new(0)),
        );
        let legion = engine.game.spawn_unit(
            UnitClass::Legion,
            Location::new(0, 1),
            PlayerId::new(0),
            Some(CityId::new(0)),
        );
        let board = engine.submit(Command::Move {
            unit: legion,
            direction: Direction::E,
        });
        assert_eq!(board[0].message(), "Unit 1 boards the ship");
        let sail = engine.submit(Command::Move {
            unit: trireme,
            direction: Direction::E,
        });
        assert!(!sail.is_empty(), "the trireme sails");
        assert_eq!(
            engine
                .game
                .units
                .iter()
                .find(|unit| unit.id() == legion)
                .unwrap()
                .location,
            Location::new(2, 1),
            "the cargo sails with its ship"
        );

        let data = SaveData::capture(&engine, Competition::new(1), Difficulty::Normal);
        let restored: SaveData =
            serde_json::from_str(&serde_json::to_string(&data).unwrap()).unwrap();
        let loaded = restored.into_loaded().unwrap();
        assert_eq!(loaded.engine.game, engine.game);

        let cargo = loaded
            .engine
            .game
            .units
            .iter()
            .find(|unit| unit.id() == legion)
            .expect("the transported unit is still there");
        let ship = loaded
            .engine
            .game
            .units
            .iter()
            .find(|unit| unit.id() == trireme)
            .expect("the trireme is still there");
        assert!(cargo.is_transported(), "the unit stays aboard");
        assert_eq!(cargo.aboard(), Some(trireme));
        assert_eq!(
            cargo.location, ship.location,
            "the cargo still shares the ship's tile"
        );
        assert!(
            loaded
                .engine
                .units_at(ship.location.x as usize, ship.location.y as usize)
                .iter()
                .all(|unit| unit.id() != legion),
            "the map still omits the transported unit"
        );
    }

    /// The rival's war declarations are transient: they are announced to the
    /// player as the round resolves, then drained, so a saved game must
    /// rebuild them empty rather than carry a stale (or re-announced) war
    /// across a save/load round trip.
    #[test]
    fn rival_war_declarations_do_not_survive_save_and_load() {
        let mut engine = engine();
        engine.rival_wars.push(RivalWar {
            rival: PlayerId::new(1),
        });
        let data = SaveData::capture(&engine, Competition::new(1), Difficulty::Normal);
        let restored: SaveData =
            serde_json::from_str(&serde_json::to_string(&data).unwrap()).unwrap();
        let mut loaded = restored.into_loaded().unwrap();
        assert!(
            loaded.engine.drain_rival_wars().is_empty(),
            "the war record is rebuilt fresh on load, never replayed"
        );
    }

    /// The build-completion queue is transient for the same reason: each entry
    /// has already been announced to the player (or is about to be) on screen,
    /// so carrying it across a save/load would re-announce a build the player
    /// has seen, or lose one they never did.
    #[test]
    fn build_completions_do_not_survive_save_and_load() {
        let mut engine = engine();
        let city = engine
            .game
            .add_city(PlayerId::new(0), "London", Location::new(2, 2));
        engine.builds.push(BuildComplete {
            city,
            city_name: "London".to_string(),
            target: ProductionTarget::Unit(UnitClass::Militia),
        });
        let data = SaveData::capture(&engine, Competition::new(1), Difficulty::Normal);
        let restored: SaveData =
            serde_json::from_str(&serde_json::to_string(&data).unwrap()).unwrap();
        let mut loaded = restored.into_loaded().unwrap();
        assert!(
            loaded.engine.drain_build_completions().is_empty(),
            "the build queue is rebuilt fresh on load, never replayed"
        );
    }

    /// A diplomat's audience is transient for the same reason the build queue
    /// is: the window it exists to open has either been answered or was never
    /// seen, so carrying it across a save/load would either re-open a choice the
    /// player already made or lose one they never did. The record is rebuilt
    /// fresh, and the diplomat's arrival into the city is history, not news.
    #[test]
    fn diplomat_audiences_do_not_survive_save_and_load() {
        let mut engine = engine();
        let city = engine
            .game
            .add_city(PlayerId::new(0), "London", Location::new(2, 2));
        engine.audiences.push(DiplomatAudience {
            unit: UnitId::new(0),
            city,
            city_name: "London".to_string(),
            from: Location::new(1, 2),
            moves_before: 2,
        });
        let data = SaveData::capture(&engine, Competition::new(1), Difficulty::Normal);
        let restored: SaveData =
            serde_json::from_str(&serde_json::to_string(&data).unwrap()).unwrap();
        let mut loaded = restored.into_loaded().unwrap();
        assert!(
            loaded.engine.drain_diplomat_audiences().is_empty(),
            "the audience queue is rebuilt fresh on load, never replayed"
        );
    }

    /// What a diplomat paid for belongs to the city, not to the diplomat: the
    /// finding must outlive the man who bought it, because he is removed the
    /// moment he acts. This is also the round trip that would catch the flag
    /// being added without `#[serde(default)]`, and so breaking every older
    /// save file.
    #[test]
    fn what_a_diplomat_learned_outlives_the_diplomat() {
        let mut engine = engine();
        engine
            .game
            .add_city(PlayerId::new(0), "London", Location::new(2, 2));
        engine.game.cities[0].mark_investigated();
        let data = SaveData::capture(&engine, Competition::new(1), Difficulty::Normal);
        let restored: SaveData =
            serde_json::from_str(&serde_json::to_string(&data).unwrap()).unwrap();
        let loaded = restored.into_loaded().unwrap();
        assert!(
            loaded.engine.game.cities[0].investigated(),
            "an investigated city stays investigated"
        );
    }

    /// `Event` carries a `#[serde(default)]` `about`, so a file written
    /// before events learned whose story they tell still loads — the old events
    /// default to `Everyone` and stay visible rather than silently vanishing.
    #[test]
    fn events_from_an_older_build_still_load_and_stay_visible() {
        let old_event: Event =
            serde_json::from_str(r#"{"message":"Roman begins turn 3"}"#).unwrap();
        assert_eq!(old_event.message(), "Roman begins turn 3");
        assert!(
            old_event.is_about(PlayerId::new(0)),
            "an event written before attribution defaults to visible"
        );
    }

    #[test]
    fn loading_a_newer_format_is_refused() {
        let engine = engine();
        let data = SaveData::capture(&engine, Competition::new(1), Difficulty::Normal);
        let json =
            serde_json::to_string(&data)
                .unwrap()
                .replacen("\"version\":1", "\"version\":999", 1);
        let parsed: SaveData = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed.into_loaded(), Err(SaveError::Version(999))));
    }
}
