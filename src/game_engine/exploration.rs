use crate::model::cartography::Location;

#[derive(Clone, Debug, PartialEq)]
pub struct Exploration {
    width: usize,
    height: usize,
    grid: Vec<Vec<bool>>,
}

impl Exploration {
    pub fn empty() -> Self {
        Exploration {
            width: 0,
            height: 0,
            grid: Vec::new(),
        }
    }

    pub fn new(width: usize, height: usize) -> Self {
        Exploration {
            width,
            height,
            grid: vec![vec![false; width]; height],
        }
    }

    pub fn marks(&self, x: usize, y: usize) -> bool {
        self.grid[y][x]
    }

    pub fn reveal(&mut self, origin: Location, radius: u8) {
        let radius = radius as isize;
        let x0 = origin.x as isize;
        let y0 = origin.y as isize;
        for y in (y0 - radius)..=(y0 + radius) {
            if y < 0 || y >= self.height as isize {
                continue;
            }
            for x in (x0 - radius)..=(x0 + radius) {
                let px = x.rem_euclid(self.width as isize) as usize;
                self.grid[y as usize][px] = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_with_nothing_explored() {
        let exploration = Exploration::new(5, 5);
        assert!(!exploration.marks(2, 2));
    }

    #[test]
    fn reveal_marks_every_tile_within_the_radius() {
        let mut exploration = Exploration::new(5, 5);
        exploration.reveal(Location::new(2, 2), 1);
        for y in 1..=3 {
            for x in 1..=3 {
                assert!(exploration.marks(x, y));
            }
        }
        assert!(!exploration.marks(0, 0));
        assert!(!exploration.marks(4, 4));
    }

    #[test]
    fn reveal_wraps_around_east_and_west_but_clamps_north_and_south() {
        let mut exploration = Exploration::new(3, 3);
        exploration.reveal(Location::new(0, 0), 1);
        for y in 0..=1 {
            for x in 0..3 {
                assert!(exploration.marks(x, y));
            }
        }
        assert!(!exploration.marks(0, 2));
        assert!(!exploration.marks(2, 2));
    }
}
