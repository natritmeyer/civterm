use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, EnumIter, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    #[default]
    Normal,
    Hard,
}

/// An exact multiplier, numerator over denominator, so the difficulty's economic
/// handicaps stay integer and visible rather than a float literal buried in a
/// call site.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scale {
    numerator: u32,
    denominator: u32,
}

impl Scale {
    /// The multiplier that changes nothing.
    pub const IDENTITY: Scale = Scale {
        numerator: 1,
        denominator: 1,
    };

    pub const fn new(numerator: u32, denominator: u32) -> Self {
        Scale {
            numerator,
            denominator,
        }
    }

    /// Apply the scale to `value`, never dropping a nonzero value to zero: a
    /// city that produced a single shield or research point must keep producing
    /// one, or a small rival city would be frozen out of building and
    /// researching entirely.
    pub fn apply(self, value: u32) -> u32 {
        let scaled = value * self.numerator / self.denominator;
        if value > 0 { scaled.max(1) } else { 0 }
    }
}

/// The three economic multipliers a difficulty imposes on a rival's cities. The
/// human's cities always run at [`Scales::IDENTITY`]; only rivals are scaled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scales {
    pub production: Scale,
    pub research: Scale,
    pub growth: Scale,
    pub gold: Scale,
}

impl Scales {
    /// The multipliers that change nothing — the human's own economy.
    pub const IDENTITY: Scales = Scales {
        production: Scale::IDENTITY,
        research: Scale::IDENTITY,
        growth: Scale::IDENTITY,
        gold: Scale::IDENTITY,
    };
}

impl Difficulty {
    pub fn display_name(&self) -> &'static str {
        match self {
            Difficulty::Easy => "Easy",
            Difficulty::Normal => "Normal",
            Difficulty::Hard => "Hard",
        }
    }

    /// The gold a civilization begins the game with. Easy stakes the human a
    /// treasury; the harder games start empty, as Civ 1 did past Chieftain.
    pub fn starting_gold(&self) -> u32 {
        match self {
            Difficulty::Easy => 50,
            Difficulty::Normal | Difficulty::Hard => 0,
        }
    }

    /// Whether rivals are willing to declare war on the human at all. On Easy
    /// they never start one — the neighbors stay peaceful and the game is a
    /// builder's game. A war the human declares is unaffected: the rival will
    /// still fight back.
    pub fn rivals_initiate_war(&self) -> bool {
        !matches!(self, Difficulty::Easy)
    }

    /// Whether a rival will attack a target of equal power. Normal insists on a
    /// strict edge; Hard settles for an even fight, which is what makes rivals
    /// so much more likely to fall on the player.
    pub fn rivals_attack_at_even_odds(&self) -> bool {
        matches!(self, Difficulty::Hard)
    }

    /// The least win chance, as a percentage, a rival demands before it
    /// commits to a fight. Hard has no floor, but note the power comparison in
    /// `rival_best_attack` is the binding constraint: any strict edge already
    /// clears a fight, so the floor only ever bites on Hard's even-odds attacks.
    pub fn rival_minimum_win_chance(&self) -> u32 {
        match self {
            Difficulty::Hard => 0,
            Difficulty::Easy | Difficulty::Normal => 10,
        }
    }

    /// The shield-output multiplier a rival's cities run at. Easy slows the
    /// rival's industry, Hard speeds it up; the human is never scaled.
    pub fn rival_production_scale(&self) -> Scale {
        match self {
            Difficulty::Easy => Scale::new(3, 4),
            Difficulty::Normal => Scale::IDENTITY,
            Difficulty::Hard => Scale::new(5, 4),
        }
    }

    /// The research-output multiplier a rival's cities run at. Easy's rivals
    /// think slowly, Hard's quickly.
    pub fn rival_research_scale(&self) -> Scale {
        match self {
            Difficulty::Easy => Scale::new(3, 4),
            Difficulty::Normal => Scale::IDENTITY,
            Difficulty::Hard => Scale::new(5, 4),
        }
    }

    /// The multiplier on the food a rival's city must store to grow — Civ 1's
    /// "rows of food". Easy widens the box (slower growth), Hard narrows it.
    pub fn rival_growth_scale(&self) -> Scale {
        match self {
            Difficulty::Easy => Scale::new(3, 2),
            Difficulty::Normal => Scale::IDENTITY,
            Difficulty::Hard => Scale::new(3, 4),
        }
    }

    /// The multiplier on the gold a rival's cities bank. Easy leaves the rival
    /// a poorer treasury, Hard a fatter one; the human is never scaled.
    pub fn rival_gold_scale(&self) -> Scale {
        match self {
            Difficulty::Easy => Scale::new(3, 4),
            Difficulty::Normal => Scale::IDENTITY,
            Difficulty::Hard => Scale::new(5, 4),
        }
    }

    /// All four rival economic multipliers, as the one value the engine hands
    /// to a player's cities.
    pub fn rival_economy(&self) -> Scales {
        Scales {
            production: self.rival_production_scale(),
            research: self.rival_research_scale(),
            growth: self.rival_growth_scale(),
            gold: self.rival_gold_scale(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn difficulty_has_three_levels_in_order() {
        let levels: Vec<Difficulty> = Difficulty::iter().collect();
        assert_eq!(
            levels,
            vec![Difficulty::Easy, Difficulty::Normal, Difficulty::Hard]
        );
    }

    #[test]
    fn each_difficulty_has_a_display_name() {
        assert_eq!(Difficulty::Easy.display_name(), "Easy");
        assert_eq!(Difficulty::Normal.display_name(), "Normal");
        assert_eq!(Difficulty::Hard.display_name(), "Hard");
    }

    #[test]
    fn normal_is_the_default() {
        assert_eq!(Difficulty::default(), Difficulty::Normal);
    }

    #[test]
    fn only_easy_stakes_the_player_starting_gold() {
        assert_eq!(Difficulty::Easy.starting_gold(), 50);
        assert_eq!(Difficulty::Normal.starting_gold(), 0);
        assert_eq!(Difficulty::Hard.starting_gold(), 0);
    }

    #[test]
    fn only_easy_holds_the_rivals_back_from_war() {
        assert!(!Difficulty::Easy.rivals_initiate_war());
        assert!(Difficulty::Normal.rivals_initiate_war());
        assert!(Difficulty::Hard.rivals_initiate_war());
    }

    #[test]
    fn only_hard_lets_rivals_attack_at_even_odds() {
        assert!(!Difficulty::Easy.rivals_attack_at_even_odds());
        assert!(!Difficulty::Normal.rivals_attack_at_even_odds());
        assert!(Difficulty::Hard.rivals_attack_at_even_odds());
    }

    #[test]
    fn only_hard_drops_the_rival_win_chance_floor() {
        assert_eq!(Difficulty::Easy.rival_minimum_win_chance(), 10);
        assert_eq!(Difficulty::Normal.rival_minimum_win_chance(), 10);
        assert_eq!(Difficulty::Hard.rival_minimum_win_chance(), 0);
    }

    #[test]
    fn identity_scale_leaves_a_value_alone() {
        assert_eq!(Scale::IDENTITY.apply(7), 7);
        assert_eq!(Scale::IDENTITY.apply(0), 0);
    }

    #[test]
    fn a_scale_truncates_the_way_integer_division_does() {
        assert_eq!(Scale::new(3, 4).apply(10), 7);
        assert_eq!(Scale::new(5, 4).apply(10), 12);
        assert_eq!(Scale::new(3, 2).apply(5), 7);
    }

    #[test]
    fn a_scale_never_cuts_a_nonzero_yield_to_nothing() {
        // A size-one rival city producing a single shield must not be frozen
        // out of building entirely, however steep the handicap.
        assert_eq!(Scale::new(1, 10).apply(1), 1);
        assert_eq!(Scale::new(3, 4).apply(0), 0);
    }

    #[test]
    fn easy_handicaps_rival_production_and_research_and_slows_growth() {
        let easy = Difficulty::Easy;
        assert_eq!(easy.rival_production_scale(), Scale::new(3, 4));
        assert_eq!(easy.rival_research_scale(), Scale::new(3, 4));
        // The food box widens: a size-two city needs 3 rows to grow, not 2.
        assert_eq!(easy.rival_growth_scale(), Scale::new(3, 2));
        assert_eq!(easy.rival_gold_scale(), Scale::new(3, 4));
    }

    #[test]
    fn hard_favours_rival_production_and_research_and_speeds_growth() {
        let hard = Difficulty::Hard;
        assert_eq!(hard.rival_production_scale(), Scale::new(5, 4));
        assert_eq!(hard.rival_research_scale(), Scale::new(5, 4));
        assert_eq!(hard.rival_growth_scale(), Scale::new(3, 4));
        assert_eq!(hard.rival_gold_scale(), Scale::new(5, 4));
    }

    #[test]
    fn normal_scales_every_rival_economy_by_identity() {
        let normal = Difficulty::Normal;
        for scale in [
            normal.rival_production_scale(),
            normal.rival_research_scale(),
            normal.rival_growth_scale(),
            normal.rival_gold_scale(),
        ] {
            assert_eq!(scale, Scale::IDENTITY);
        }
        assert_eq!(normal.rival_economy(), Scales::IDENTITY);
    }
}
