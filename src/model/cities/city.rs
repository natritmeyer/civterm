use crate::model::cartography::Location;
use crate::model::cities::{CityId, CityImprovement, CityTick, ProductionTarget};
use crate::model::civilizations::PlayerId;

#[derive(Clone, Debug, PartialEq)]
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
            worked: Vec::new(),
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

    pub fn grow(&mut self) {
        self.population += 1;
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

        let starving = net_food < 0 && self.food == 0;
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
        assert!(city.worked_tiles().is_empty());
        city.add_worked_tile(Location::new(1, 1));
        city.add_worked_tile(Location::new(1, 1));
        city.add_worked_tile(Location::new(0, 1));
        assert_eq!(city.worked_tiles().len(), 2);
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
