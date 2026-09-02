use crate::game_engine::Exploration;
use crate::model::advancements::Advancement;
use crate::model::cartography::Location;
use crate::model::cities::{CityImprovement, ProductionTarget};
use crate::model::civilizations::{Civilization, PlayerId};
use crate::model::units::UnitClass;
use strum::IntoEnumIterator;

/// The gold a civilization begins a new game with.
pub const STARTING_GOLD: u32 = 50;

#[derive(Clone, Debug, PartialEq)]
pub struct Player {
    pub civilization: Civilization,
    gold: u32,
    advancement_in_progress: Option<Advancement>,
    research_progress: u32,
    advances_made: Vec<Advancement>,
    explored: Exploration,
    pub(super) at_war_with: Vec<PlayerId>,
    pub(super) at_peace_with: Vec<PlayerId>,
}

impl Player {
    pub fn new(civilization: Civilization) -> Self {
        Player {
            civilization,
            gold: STARTING_GOLD,
            advancement_in_progress: None,
            research_progress: 0,
            advances_made: Vec::new(),
            explored: Exploration::empty(),
            at_war_with: Vec::new(),
            at_peace_with: Vec::new(),
        }
    }

    pub fn gold(&self) -> u32 {
        self.gold
    }

    pub fn at_war_with(&self) -> &[PlayerId] {
        &self.at_war_with
    }

    pub fn at_peace_with(&self) -> &[PlayerId] {
        &self.at_peace_with
    }

    pub(super) fn enter_war_with(&mut self, other: PlayerId) {
        if !self.at_war_with.contains(&other) {
            self.at_war_with.push(other);
        }
        self.at_peace_with.retain(|player| *player != other);
    }

    pub(super) fn enter_peace_with(&mut self, other: PlayerId) {
        if !self.at_peace_with.contains(&other) {
            self.at_peace_with.push(other);
        }
        self.at_war_with.retain(|player| *player != other);
    }

    pub fn advancement_in_progress(&self) -> Option<Advancement> {
        self.advancement_in_progress
    }

    pub fn advances_made(&self) -> &[Advancement] {
        &self.advances_made
    }

    /// Begin research on Construction unless a target is already in progress.
    pub(super) fn begin_research(&mut self) {
        if self.advancement_in_progress.is_none() {
            self.advancement_in_progress = Some(Advancement::Construction);
            self.research_progress = 0;
        }
    }

    pub fn research_progress(&self) -> u32 {
        self.research_progress
    }

    /// Set the advancement being researched, resetting accumulated progress.
    /// No validation (prerequisites etc.) is performed here; the engine guards.
    pub(super) fn set_research_target(&mut self, advancement: Advancement) {
        self.advancement_in_progress = Some(advancement);
        self.research_progress = 0;
    }

    /// Add this turn's beakers from all cities. When the current target's cost
    /// is reached, the advancement is discovered and returned.
    pub(super) fn advance_research(&mut self, beakers: u32) -> Option<Advancement> {
        let target = self.advancement_in_progress?;
        self.research_progress += beakers;
        if self.research_progress >= target.cost() {
            self.advances_made.push(target);
            self.advancement_in_progress = None;
            self.research_progress = 0;
            Some(target)
        } else {
            None
        }
    }

    pub fn has_advancement(&self, advancement: Advancement) -> bool {
        self.advances_made.contains(&advancement)
    }

    /// Advancements the player may begin researching next: those not yet
    /// discovered whose prerequisites have all been discovered.
    pub fn researchable_advancements(&self) -> Vec<Advancement> {
        Advancement::iter()
            .filter(|adv| {
                !self.has_advancement(*adv)
                    && adv
                        .prerequisites()
                        .iter()
                        .all(|prereq| self.has_advancement(*prereq))
            })
            .collect()
    }

    /// Whether this player may currently build the given production target,
    /// i.e. the target's required advancement (if any) has been discovered.
    pub fn can_build(&self, target: ProductionTarget) -> bool {
        match target.required_advancement() {
            Some(adv) => self.has_advancement(adv),
            None => true,
        }
    }

    /// Unit classes available to this player based on their discovered
    /// advancements (those with no requirement, plus those whose required
    /// advancement has been discovered).
    pub fn available_unit_classes(&self) -> Vec<UnitClass> {
        UnitClass::iter()
            .filter(|class| match class.required_advancement() {
                Some(adv) => self.has_advancement(adv),
                None => true,
            })
            .collect()
    }

    /// Improvements available to this player based on their discovered
    /// advancements.
    pub fn available_improvements(&self) -> Vec<CityImprovement> {
        CityImprovement::iter()
            .filter(|imp| match imp.required_advancement() {
                Some(adv) => self.has_advancement(adv),
                None => true,
            })
            .collect()
    }

    pub(super) fn seed_exploration(&mut self, width: usize, height: usize) {
        self.explored = Exploration::new(width, height);
    }

    pub fn explored_at(&self, x: usize, y: usize) -> bool {
        self.explored.discovered(x, y)
    }

    pub fn reveal_tiles_at(&mut self, origin: Location, radius: u8) {
        self.explored.reveal_tiles_at(origin, radius);
    }

    pub fn reveal_tiles_surrounding_city_at(&mut self, origin: Location) {
        self.explored.reveal_tiles_surrounding_city_at(origin);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_is_created_with_given_civilization() {
        let player = Player::new(Civilization::English);
        assert_eq!(player.civilization, Civilization::English);
    }

    #[test]
    fn player_starts_with_no_advancement_in_progress() {
        let player = Player::new(Civilization::English);
        assert_eq!(player.advancement_in_progress(), None);
    }

    #[test]
    fn player_starts_with_no_advances_made() {
        let player = Player::new(Civilization::English);
        assert!(player.advances_made().is_empty());
    }

    #[test]
    fn beginning_research_targets_construction() {
        let mut player = Player::new(Civilization::English);
        player.begin_research();
        assert_eq!(
            player.advancement_in_progress(),
            Some(Advancement::Construction)
        );
        assert_eq!(player.research_progress(), 0);
    }

    #[test]
    fn beginning_research_preserves_an_existing_target() {
        let mut player = Player::new(Civilization::English);
        player.set_research_target(Advancement::Wheel);
        player.begin_research();
        assert_eq!(player.advancement_in_progress(), Some(Advancement::Wheel));
    }

    #[test]
    fn setting_research_target_resets_progress() {
        let mut player = Player::new(Civilization::English);
        player.begin_research();
        player.advance_research(50);
        assert_eq!(player.research_progress(), 50);
        player.set_research_target(Advancement::Wheel);
        assert_eq!(player.research_progress(), 0);
        assert_eq!(player.advancement_in_progress(), Some(Advancement::Wheel));
    }

    #[test]
    fn research_accumulates_towards_the_target_and_discovers_it_at_cost() {
        let mut player = Player::new(Civilization::English);
        player.set_research_target(Advancement::Wheel); // cost 40
        assert_eq!(player.advance_research(30), None);
        assert_eq!(player.research_progress(), 30);
        assert_eq!(player.advance_research(10), Some(Advancement::Wheel));
        assert!(player.has_advancement(Advancement::Wheel));
        assert_eq!(player.advancement_in_progress(), None);
        assert_eq!(player.research_progress(), 0);
    }

    #[test]
    fn research_before_any_target_does_nothing() {
        let mut player = Player::new(Civilization::English);
        assert_eq!(player.advance_research(100), None);
        assert_eq!(player.research_progress(), 0);
    }

    #[test]
    fn researchable_advancements_starts_with_the_founding_techs() {
        let player = Player::new(Civilization::English);
        let available = player.researchable_advancements();
        for founding in [
            Advancement::Alphabet,
            Advancement::BronzeWorking,
            Advancement::CeremonialBurial,
            Advancement::HorsebackRiding,
            Advancement::Masonry,
            Advancement::Pottery,
            Advancement::Wheel,
        ] {
            assert!(available.contains(&founding));
        }
        // Deeper techs require undiscovered prerequisites.
        assert!(!available.contains(&Advancement::Engineering));
        assert!(!available.contains(&Advancement::Astronomy));
    }

    #[test]
    fn researchable_advancements_unlock_as_prerequisites_are_discovered() {
        let mut player = Player::new(Civilization::English);
        // Discover Wheel (a founding tech); Currency now requires BronzeWorking.
        player.set_research_target(Advancement::Wheel);
        player.advance_research(999);
        assert!(
            !player
                .researchable_advancements()
                .contains(&Advancement::Currency)
        );
        // Discover BronzeWorking, which unlocks Currency.
        player.set_research_target(Advancement::BronzeWorking);
        player.advance_research(999);
        assert!(
            player
                .researchable_advancements()
                .contains(&Advancement::Currency)
        );
    }

    #[test]
    fn a_player_has_met_nobody_at_the_start() {
        let player = Player::new(Civilization::English);
        assert!(player.at_war_with().is_empty());
        assert!(player.at_peace_with().is_empty());
    }

    #[test]
    fn entering_war_removes_a_previous_peace() {
        let mut player = Player::new(Civilization::English);
        player.enter_peace_with(PlayerId::new(1));
        player.enter_war_with(PlayerId::new(1));
        assert!(player.at_war_with().contains(&PlayerId::new(1)));
        assert!(!player.at_peace_with().contains(&PlayerId::new(1)));
    }

    #[test]
    fn entering_peace_removes_a_previous_war() {
        let mut player = Player::new(Civilization::English);
        player.enter_war_with(PlayerId::new(1));
        player.enter_peace_with(PlayerId::new(1));
        assert!(!player.at_war_with().contains(&PlayerId::new(1)));
        assert!(player.at_peace_with().contains(&PlayerId::new(1)));
    }

    #[test]
    fn can_build_allows_units_with_no_required_advancement() {
        let player = Player::new(Civilization::English);
        assert!(player.can_build(ProductionTarget::Unit(UnitClass::Militia)));
        assert!(player.can_build(ProductionTarget::Unit(UnitClass::Settler)));
    }

    #[test]
    fn can_build_rejects_units_requiring_an_undiscovered_advancement() {
        let player = Player::new(Civilization::English);
        assert!(!player.can_build(ProductionTarget::Unit(UnitClass::Knight)));
        assert!(!player.can_build(ProductionTarget::Unit(UnitClass::Legion)));
        assert!(!player.can_build(ProductionTarget::Unit(UnitClass::Trireme)));
    }

    #[test]
    fn can_build_allows_units_once_the_required_advancement_is_discovered() {
        let mut player = Player::new(Civilization::English);
        player.set_research_target(Advancement::Chivalry);
        player.advance_research(999);
        assert!(player.can_build(ProductionTarget::Unit(UnitClass::Knight)));
    }

    #[test]
    fn can_build_rejects_improvements_requiring_undiscovered_advancement() {
        let player = Player::new(Civilization::English);
        assert!(!player.can_build(ProductionTarget::Improvement(CityImprovement::Library)));
    }

    #[test]
    fn available_unit_classes_excludes_gated_units() {
        let player = Player::new(Civilization::English);
        let available = player.available_unit_classes();
        assert!(available.contains(&UnitClass::Militia));
        assert!(available.contains(&UnitClass::Settler));
        assert!(!available.contains(&UnitClass::Knight));
        assert!(!available.contains(&UnitClass::Frigate));
        assert!(!available.contains(&UnitClass::Trireme));
    }

    #[test]
    fn available_improvements_excludes_gated_improvements() {
        let player = Player::new(Civilization::English);
        let available = player.available_improvements();
        assert!(available.contains(&CityImprovement::Barracks));
        assert!(!available.contains(&CityImprovement::Library));
        assert!(!available.contains(&CityImprovement::Bank));
    }

    #[test]
    fn available_unit_classes_grows_with_discovered_advancements() {
        let mut player = Player::new(Civilization::English);
        assert!(!player.available_unit_classes().contains(&UnitClass::Knight));
        player.set_research_target(Advancement::Chivalry);
        player.advance_research(999);
        assert!(player.available_unit_classes().contains(&UnitClass::Knight));
    }
}
