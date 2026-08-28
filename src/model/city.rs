use crate::model::cartography::Location;

#[derive(Clone, Debug, PartialEq)]
pub struct City {
    pub name: String,
    pub location: Location,
    population: u32,
}

impl City {
    pub fn new(name: impl Into<String>, location: Location) -> Self {
        City {
            name: name.into(),
            location,
            population: 1,
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn city_is_created_with_name_and_location() {
        let location = Location::new(3, 4);
        let city = City::new("London", location);
        assert_eq!(city.name, "London");
        assert_eq!(city.location, location);
    }

    #[test]
    fn city_starts_with_population_1() {
        let city = City::new("London", Location::new(0, 0));
        assert_eq!(city.population(), 1);
    }

    #[test]
    fn growing_a_city_increments_its_population() {
        let mut city = City::new("London", Location::new(0, 0));
        city.grow();
        city.grow();
        assert_eq!(city.population(), 3);
    }

    #[test]
    fn shrinking_a_city_decrements_its_population() {
        let mut city = City::new("London", Location::new(0, 0));
        city.grow();
        city.shrink();
        assert_eq!(city.population(), 1);
    }

    #[test]
    fn shrinking_does_not_go_below_one() {
        let city = City::new("London", Location::new(0, 0));
        let mut city = city;
        city.shrink();
        city.shrink();
        assert_eq!(city.population(), 1);
    }
}
