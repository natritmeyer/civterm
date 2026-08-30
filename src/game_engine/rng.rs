#[derive(Clone, Debug)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng {
            state: if seed == 0 {
                0x9E37_79B9_7F4A_7C15
            } else {
                seed
            },
        }
    }

    pub fn in_range(&mut self, max: u32) -> u32 {
        (self.next_u64() % max as u64) as u32
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_range_is_below_the_max() {
        for _ in 0..100 {
            let mut rng = Rng::new(7);
            assert!(rng.in_range(30) < 30);
        }
    }

    #[test]
    fn same_seed_produces_the_same_sequence() {
        let mut first = Rng::new(42);
        let mut second = Rng::new(42);
        for _ in 0..50 {
            assert_eq!(first.in_range(1000), second.in_range(1000));
        }
    }

    #[test]
    fn different_seeds_produce_different_sequences() {
        let mut first = Rng::new(1);
        let mut second = Rng::new(2);
        let mut identical = true;
        for _ in 0..50 {
            if first.in_range(1000) != second.in_range(1000) {
                identical = false;
                break;
            }
        }
        assert!(!identical);
    }
}
