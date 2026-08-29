#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UnitId(usize);

impl UnitId {
    pub fn new(index: usize) -> Self {
        UnitId(index)
    }

    pub fn index(&self) -> usize {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_id_wraps_an_index() {
        assert_eq!(UnitId::new(3).index(), 3);
    }
}
