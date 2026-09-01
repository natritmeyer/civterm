use crate::model::cartography::Map;
use crate::utils::Rng;

pub struct MapGenerator {
    rng: Rng,
}

impl MapGenerator {
    pub fn new(seed: u64) -> Self {
        MapGenerator {
            rng: Rng::new(seed),
        }
    }

    pub fn with_rng(rng: Rng) -> Self {
        MapGenerator { rng }
    }

    pub fn rng(&self) -> &Rng {
        &self.rng
    }
}
