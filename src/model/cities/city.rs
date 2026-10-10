use crate::model::cartography::Location;
use crate::model::cities::{CityId, CityImprovement, CityTick, ProductionTarget};
use crate::model::civilizations::PlayerId;
use serde::{Deserialize, Serialize};

/// The largest a city can ever be. Growth stops here rather than running on,
/// which is also why the map's city tile can afford two digits for it.
pub const MAX_POPULATION: u32 = 99;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct City {
    pub name: String,
    pub location: Location,
    id: CityId,
    owner: PlayerId,
    population: u32,
    food: u32,
    resources: u32,
    trade: u32,
    improvements: Vec<CityImprovement>,
    production: Option<ProductionTarget>,
    resource_stored: u32,
    worked: Vec<Location>,
    /// Whether a foreign diplomat has paid to look inside this city. The
    /// knowledge belongs to whoever paid for it, but the city is the only
    /// place to keep the fact, and only a rival's city can be true.
    #[serde(default)]
    investigated: bool,
}

impl City {
    pub fn new(name: impl Into<String>, location: Location, owner: PlayerId, id: CityId) -> Self {
        City {
            name: name.into(),
            location,
            id,
            owner,
            population: 1,
            food: 0,
            resources: 0,
            trade: 0,
            improvements: Vec::new(),
            production: None,
            resource_stored: 0,
            investigated: false,
            // The city centre is always worked from the moment it is founded.
            worked: vec![location],
        }
    }

    pub fn id(&self) -> CityId {
        self.id
    }

    pub fn owner(&self) -> PlayerId {
        self.owner
    }

    pub fn change_owner(&mut self, owner: PlayerId) {
        self.owner = owner;
    }

    pub fn population(&self) -> u32 {
        self.population
    }

    /// Grows the city by one, stopping at [`MAX_POPULATION`].
    pub fn grow(&mut self) {
        self.population = self.population.saturating_add(1).min(MAX_POPULATION);
    }

    pub fn shrink(&mut self) {
        self.population = self.population.saturating_sub(1).max(1);
    }

    pub fn food(&self) -> u32 {
        self.food
    }

    pub fn resources(&self) -> u32 {
        self.resources
    }

    pub fn trade(&self) -> u32 {
        self.trade
    }

    pub fn improvements(&self) -> &[CityImprovement] {
        &self.improvements
    }

    pub fn add_improvement(&mut self, improvement: CityImprovement) {
        self.improvements.push(improvement);
    }

    /// Tear out an improvement, as a catapult does to a city wall. Returns
    /// whether it was there to be torn out, so the caller knows if the shot
    /// found anything to hit.
    /// Whether a diplomat has already looked inside this city.
    pub fn investigated(&self) -> bool {
        self.investigated
    }

    /// Record that a diplomat has looked inside this city, and hand back
    /// whether this was the first time — the TUI only ever renders the report
    /// window once the flag is set, but the purchase itself is never barred.
    pub fn mark_investigated(&mut self) -> bool {
        let first_time = !self.investigated;
        self.investigated = true;
        first_time
    }

    pub fn remove_improvement(&mut self, improvement: CityImprovement) -> bool {
        let before = self.improvements.len();
        self.improvements.retain(|held| *held != improvement);
        self.improvements.len() != before
    }
    pub fn set_production(&mut self, target: ProductionTarget) {
        self.production = Some(target);
        self.resource_stored = 0;
    }

    pub fn worked_tiles(&self) -> &[Location] {
        &self.worked
    }

    pub fn add_worked_tile(&mut self, location: Location) {
        if !self.worked.contains(&location) {
            self.worked.push(location);
        }
    }

    /// Food consumed each turn: each citizen eats 2.
    pub fn food_consumption(&self) -> u32 {
        2 * self.population
    }

    /// The food surplus the city stores before its population grows.
    pub fn growth_target(&self) -> u32 {
        2 * self.population
    }

    /// Advance the city one turn given this turn's food and resource income.
    pub fn tick(&mut self, food_income: u32, resource_income: u32) -> CityTick {
        let consumption = self.food_consumption();
        let net_food = food_income as i32 - consumption as i32;
        let produced = resource_income;

        let food_deficit = (-net_food).max(0) as u32;
        let food_surplus = net_food.max(0) as u32;

        self.food = self
            .food
            .saturating_add(food_surplus)
            .saturating_sub(food_deficit);
        self.resources = self.resources.saturating_add(produced);
        self.resource_stored = self.resource_stored.saturating_add(produced);

        let growth_need = self.population * 2;
        let mut grew = false;
        if net_food >= 0 && self.food >= growth_need {
            self.food -= growth_need;
            self.grow();
            grew = true;
        }

        // A city with an empty granary and a food deficit is starving. The last
        // citizen is never taken — a city cannot starve out of existence — so a
        // size-one city needs the warning but simply survives on nothing, and
        // `lost_citizen` stays false.
        let starving = net_food < 0 && self.food == 0;
        let lost_citizen = starving && self.population > 1;
        if lost_citizen {
            self.shrink();
        }
        let completed = match self.production {
            Some(target) if self.resource_stored >= target.resource_cost() => {
                self.resource_stored = 0;
                self.production = None;
                match target {
                    ProductionTarget::Improvement(improvement) => {
                        self.add_improvement(improvement);
                        Some(target)
                    }
                    ProductionTarget::Unit(_) => Some(target),
                }
            }
            _ => None,
        };

        CityTick {
            produced,
            grew,
            completed,
            starving,
            lost_citizen,
        }
    }

    pub fn production_target(&self) -> Option<ProductionTarget> {
        self.production
    }

    pub fn resource_stored(&self) -> u32 {
        self.resource_stored
    }

    pub fn research(&self) -> u32 {
        let mut numerator = 4u32;
        let mut denominator = 4u32;
        if self.improvements.contains(&CityImprovement::Library) {
            numerator *= 3;
            denominator *= 2;
        }
        if self.improvements.contains(&CityImprovement::University) {
            numerator *= 3;
            denominator *= 2;
        }
        self.population * numerator / denominator
    }

    /// The city's food harvest on top of the raw worked-tile yield. A Granary
    /// doubles the harvest and an Aqueduct adds half again to it, so a grown
    /// city can feed more mouths and grow sooner.
    pub fn food_income(&self, raw: u32) -> u32 {
        let mut numerator = 1u32;
        let mut denominator = 1u32;
        if self.improvements.contains(&CityImprovement::Granary) {
            numerator *= 2;
        }
        if self.improvements.contains(&CityImprovement::Aqueduct) {
            numerator *= 3;
            denominator *= 2;
        }
        raw * numerator / denominator
    }

    /// Gold minted on top of the raw special-resource yield. A Marketplace
    /// adds half again to it and a Bank adds half again to that.
    pub fn gold_income(&self, raw: u32) -> u32 {
        let mut numerator = 1u32;
        let mut denominator = 1u32;
        if self.improvements.contains(&CityImprovement::Marketplace) {
            numerator *= 3;
            denominator *= 2;
        }
        if self.improvements.contains(&CityImprovement::Bank) {
            numerator *= 3;
            denominator *= 2;
        }
        raw * numerator / denominator
    }

    /// How many of the city's citizens the culture improvements make happy.
    /// Every citizen starts content; Temple, Colosseum and Cathedral lift
    /// citizens into the happy tier, but never more than the population.
    pub fn happy_citizens(&self) -> u32 {
        let bonus: u32 = self
            .improvements
            .iter()
            .map(CityImprovement::happiness_bonus)
            .sum();
        bonus.min(self.population)
    }

    /// The citizens left content once the happy ones are taken out.
    pub fn content_citizens(&self) -> u32 {
        self.population - self.happy_citizens()
    }

    /// The city's shield harvest on top of the raw worked-tile yield. A happy
    /// citizen is twice as productive as a content one, so the raw resources
    /// are scaled by `(content + 2 * happy) / population`.
    pub fn resource_income(&self, raw: u32) -> u32 {
        let population = self.population.max(1);
        raw * (self.content_citizens() + 2 * self.happy_citizens()) / population
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::units::UnitClass;

    #[test]
    fn city_is_created_with_name_and_location() {
        let location = Location::new(3, 4);
        let id = CityId::new(2);
        let city = City::new("London", location, PlayerId::new(0), id);
        assert_eq!(city.name, "London");
        assert_eq!(city.location, location);
        assert_eq!(city.owner(), PlayerId::new(0));
        assert_eq!(city.id(), id);
    }

    #[test]
    fn city_starts_with_population_1() {
        let city = City::new(
            "London",
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
        );
        assert_eq!(city.population(), 1);
    }

    #[test]
    fn city_starts_with_zero_food_resources_and_trade() {
        let city = City::new(
            "London",
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
        );
        assert_eq!(city.food(), 0);
        assert_eq!(city.resources(), 0);
        assert_eq!(city.trade(), 0);
    }

    #[test]
    fn city_starts_with_no_improvements_or_production() {
        let city = City::new(
            "London",
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
        );
        assert!(city.improvements().is_empty());
        assert_eq!(city.production_target(), None);
    }

    #[test]
    fn growing_a_city_increments_its_population() {
        let mut city = City::new(
            "London",
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
        );
        city.grow();
        city.grow();
        assert_eq!(city.population(), 3);
    }

    #[test]
    fn a_city_stops_growing_at_the_maximum_population() {
        let mut city = City::new(
            "London",
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
        );
        for _ in 0..500 {
            city.grow();
        }
        assert_eq!(city.population(), MAX_POPULATION);
    }

    #[test]
    fn the_maximum_population_is_two_digits() {
        // The map's city tile is two columns wide; a third digit would spill
        // into the neighbouring tile, so the cap is what keeps the tile honest.
        assert_eq!(MAX_POPULATION.to_string().chars().count(), 2);
    }

    #[test]
    fn shrinking_a_city_decrements_its_population() {
        let mut city = City::new(
            "London",
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
        );
        city.grow();
        city.shrink();
        assert_eq!(city.population(), 1);
    }

    #[test]
    fn shrinking_does_not_go_below_one() {
        let city = City::new(
            "London",
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
        );
        let mut city = city;
        city.shrink();
        city.shrink();
        assert_eq!(city.population(), 1);
    }

    #[test]
    fn research_equals_population_without_improvements() {
        let city = City::new(
            "London",
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
        );
        let mut city = city;
        city.grow();
        city.grow();
        city.grow();
        assert_eq!(city.population(), 4);
        assert_eq!(city.research(), 4);
    }

    #[test]
    fn library_increases_research_by_50_percent() {
        let mut city = City::new(
            "London",
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
        );
        city.grow();
        city.grow();
        city.grow();
        city.add_improvement(CityImprovement::Library);
        assert_eq!(city.research(), 6);
    }

    #[test]
    fn university_increases_research_by_50_percent() {
        let mut city = City::new(
            "London",
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
        );
        city.grow();
        city.grow();
        city.grow();
        city.add_improvement(CityImprovement::University);
        assert_eq!(city.research(), 6);
    }

    #[test]
    fn library_and_university_multipliers_stack() {
        let mut city = City::new(
            "London",
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
        );
        city.grow();
        city.grow();
        city.grow();
        city.add_improvement(CityImprovement::Library);
        city.add_improvement(CityImprovement::University);
        assert_eq!(city.research(), 9);
    }

    #[test]
    fn granary_doubles_food_income() {
        let mut city = City::new(
            "London",
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
        );
        assert_eq!(city.food_income(4), 4);
        city.add_improvement(CityImprovement::Granary);
        assert_eq!(city.food_income(4), 8);
    }

    #[test]
    fn aqueduct_adds_half_again_food_income() {
        let mut city = City::new(
            "London",
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
        );
        city.add_improvement(CityImprovement::Aqueduct);
        assert_eq!(city.food_income(4), 6);
    }

    #[test]
    fn granary_and_aqueduct_food_bonuses_stack() {
        let mut city = City::new(
            "London",
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
        );
        city.add_improvement(CityImprovement::Granary);
        city.add_improvement(CityImprovement::Aqueduct);
        assert_eq!(city.food_income(4), 12);
    }

    #[test]
    fn marketplace_and_bank_gold_bonuses_stack() {
        let mut city = City::new(
            "London",
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
        );
        assert_eq!(city.gold_income(2), 2);
        city.add_improvement(CityImprovement::Marketplace);
        assert_eq!(city.gold_income(2), 3);
        city.add_improvement(CityImprovement::Bank);
        assert_eq!(city.gold_income(2), 4);
    }

    #[test]
    fn culture_improvements_make_citizens_happy_without_exceeding_population() {
        let mut city = City::new(
            "London",
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
        );
        // A fresh city is a single content citizen.
        assert_eq!(city.happy_citizens(), 0);
        assert_eq!(city.content_citizens(), 1);

        // A Temple makes one citizen happy.
        city.add_improvement(CityImprovement::Temple);
        assert_eq!(city.happy_citizens(), 1);
        assert_eq!(city.content_citizens(), 0);

        // Grow to seven: 1 + 3 + 4 = 8 worth of happiness, capped at seven.
        for _ in 0..6 {
            city.grow();
        }
        city.add_improvement(CityImprovement::Colosseum);
        city.add_improvement(CityImprovement::Cathedral);
        assert_eq!(city.population(), 7);
        assert_eq!(
            city.happy_citizens(),
            7,
            "happy can never exceed population"
        );
        assert_eq!(city.content_citizens(), 0);
    }

    #[test]
    fn happy_citizens_double_the_citys_production() {
        let mut city = City::new(
            "London",
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
        );
        // Grow to seven: seven content citizens, base production.
        for _ in 1..7 {
            city.grow();
        }
        assert_eq!(city.resource_income(7), 7);
        // Three of them happy: 4 content + 3 happy * 2 = 10.
        city.add_improvement(CityImprovement::Colosseum);
        assert_eq!(city.resource_income(7), 10);
    }

    #[test]
    fn food_consumption_scales_with_population() {
        let mut city = City::new(
            "London",
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
        );
        assert_eq!(city.food_consumption(), 2);
        city.grow();
        city.grow();
        assert_eq!(city.food_consumption(), 6);
    }

    #[test]
    fn worked_tiles_accumulate_and_deduplicate() {
        let mut city = City::new(
            "London",
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
        );
        assert_eq!(city.worked_tiles(), &[Location::new(0, 0)]);
        city.add_worked_tile(Location::new(1, 1));
        city.add_worked_tile(Location::new(1, 1));
        city.add_worked_tile(Location::new(0, 1));
        assert_eq!(city.worked_tiles().len(), 3);
    }

    #[test]
    fn a_city_grows_when_food_surplus_accumulates() {
        let mut city = City::new(
            "London",
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
        );
        // Pop 1 consuming 2 food, income 3 => +1 surplus per turn. Growth needs 2.
        let raised = city.tick(3, 1);
        assert!(!raised.grew);
        let raised = city.tick(3, 1);
        assert!(raised.grew);
        assert_eq!(city.population(), 2);
    }

    #[test]
    fn a_starving_city_loses_a_citizen() {
        let mut city = City::new(
            "London",
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
        );
        for _ in 0..3 {
            city.grow();
        }
        assert_eq!(city.population(), 4);
        // Pop 4 consumes 8 food; an income of 6 leaves a deficit with an empty
        // granary, so a citizen starves.
        let tick = city.tick(6, 1);
        assert!(tick.starving);
        assert!(tick.lost_citizen);
        assert_eq!(city.population(), 3);
        assert_eq!(city.food(), 0);
    }

    #[test]
    fn a_size_one_city_cannot_starve_out_of_existence() {
        let mut city = City::new(
            "London",
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
        );
        let tick = city.tick(0, 0);
        assert!(tick.starving, "a one-city still reports the deficit");
        assert!(!tick.lost_citizen, "the last citizen is never taken");
        assert_eq!(city.population(), 1);
    }

    #[test]
    fn resource_income_accrues_into_build_progress() {
        let mut city = City::new(
            "London",
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
        );
        city.set_production(ProductionTarget::Unit(UnitClass::Militia));
        // 3 resources per turn; Militia costs 10. After 3 turns stored = 9,
        // the 4th turn brings it to 12 which completes.
        let raised = city.tick(0, 3);
        assert_eq!(raised.completed, None);
        let raised = city.tick(0, 3);
        assert_eq!(raised.completed, None);
        let raised = city.tick(0, 3);
        assert_eq!(raised.completed, None);
        let raised = city.tick(0, 3);
        assert_eq!(
            raised.completed,
            Some(ProductionTarget::Unit(UnitClass::Militia))
        );
        assert_eq!(city.resource_stored(), 0);
    }

    #[test]
    fn setting_production_stores_the_target_and_resets_accumulated_resources() {
        let mut city = City::new(
            "London",
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
        );
        city.set_production(ProductionTarget::Unit(UnitClass::Militia));
        assert_eq!(
            city.production_target(),
            Some(ProductionTarget::Unit(UnitClass::Militia))
        );
        assert_eq!(city.resource_stored(), 0);
        city.set_production(ProductionTarget::Improvement(CityImprovement::Library));
        assert_eq!(
            city.production_target(),
            Some(ProductionTarget::Improvement(CityImprovement::Library))
        );
        assert_eq!(city.resource_stored(), 0);
    }
}
