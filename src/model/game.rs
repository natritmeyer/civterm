use crate::model::map::Map;

pub struct Game {
    pub map: Map,
}

impl Game {
    pub fn new(width: usize, height: usize) -> Self {
        Game {
            map: Map::new(width, height),
        }
    }
}
