use crate::model::cartography::Location;
use crate::model::cities::{CityId, CityImprovement};
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
    improvement_in_progress: Option<CityImprovement>,
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
            improvement_in_progress: None,
        }
    }

    pub fn id(&self) -> CityId {
        self.id
    }

    pub fn owner(&self) -> PlayerId {
        self.owner
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

    pub fn improvement_in_progress(&self) -> Option<CityImprovement> {
        self.improvement_in_progress
    }

    pub fn add_improvement(&mut self, improvement: CityImprovement) {
        self.improvements.push(improvement);
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
        assert_eq!(city.improvement_in_progress(), None);
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
}
