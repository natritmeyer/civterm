#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PlayerId(usize);

impl PlayerId {
    pub fn new(index: usize) -> Self {
        PlayerId(index)
    }

    pub fn index(&self) -> usize {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_id_wraps_an_index() {
        assert_eq!(PlayerId::new(3).index(), 3);
    }
}
