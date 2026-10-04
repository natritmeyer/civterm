use serde::{Deserialize, Serialize};

use crate::model::civilizations::PlayerId;

/// Who an event is about. The TUI shows the log of the civilization the player
/// commands, so an event has to say whose story it is telling: a rival founding
/// a city is the rival's news, while the same city falling to the player is
/// very much theirs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventAbout {
    /// Not about any one civilization — a rule rejection, a failure the player
    /// caused. Shown whoever is playing.
    #[default]
    Everyone,
    /// About exactly one civilization.
    Player(PlayerId),
    /// About a clash between two, shown to both.
    Both(PlayerId, PlayerId),
}

impl EventAbout {
    /// Whether the event belongs in this player's log.
    pub fn includes(self, player: PlayerId) -> bool {
        match self {
            EventAbout::Everyone => true,
            EventAbout::Player(one) => one == player,
            EventAbout::Both(a, b) => a == player || b == player,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    message: String,
    /// Whose story this is. Defaults to `Everyone` so events written by older
    /// builds — and the plain rule rejections — stay visible.
    #[serde(default)]
    about: EventAbout,
}

impl Event {
    /// An event about no particular civilization: a rejected command, or
    /// something every player should see.
    pub fn new(message: impl Into<String>) -> Self {
        Event {
            message: message.into(),
            about: EventAbout::Everyone,
        }
    }

    /// An event about one civilization's own affairs — its unit moving, its
    /// city producing, its research advancing.
    pub fn for_player(player: PlayerId, message: impl Into<String>) -> Self {
        Event {
            message: message.into(),
            about: EventAbout::Player(player),
        }
    }

    /// An event about two civilizations meeting: a war declaration, a battle,
    /// a first contact. Both players are told.
    pub fn between(one: PlayerId, other: PlayerId, message: impl Into<String>) -> Self {
        Event {
            message: message.into(),
            about: EventAbout::Both(one, other),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    /// Whether this event belongs in the given player's log.
    pub fn is_about(&self, player: PlayerId) -> bool {
        self.about.includes(player)
    }
}
