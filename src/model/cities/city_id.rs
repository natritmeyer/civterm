#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CityId(usize);

impl CityId {
    pub fn new(index: usize) -> Self {
        CityId(index)
    }

    pub fn index(&self) -> usize {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn city_id_wraps_an_index() {
        assert_eq!(CityId::new(3).index(), 3);
    }
}
