use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::game_engine::Engine;
use crate::game_engine::event::Event;
use crate::game_engine::game::Game;
use crate::model::civilizations::PlayerId;
use crate::model::competition::Competition;
use crate::model::difficulty::Difficulty;
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
    use crate::game_engine::{GameView, Player};
    use crate::model::cartography::{Direction, Location};
    use crate::model::cities::ProductionTarget;
    use crate::model::civilizations::{Civilization, PlayerId};
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
