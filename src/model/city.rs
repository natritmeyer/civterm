use crate::model::cartography::Location;

#[derive(Clone, Debug, PartialEq)]
pub struct City {
    pub name: String,
    pub location: Location,
}

impl City {
    pub fn new(name: impl Into<String>, location: Location) -> Self {
        City {
            name: name.into(),
            location,
        }
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
}
