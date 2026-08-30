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

    pub fn discovered(&self, x: usize, y: usize) -> bool {
        self.grid[y][x]
    }

    pub fn reveal_tiles_at(&mut self, origin: Location, radius: u8) {
        self.reveal_with_extent(origin, radius as isize, |_, _| true);
    }

    pub fn reveal_tiles_surrounding_city_at(&mut self, origin: Location) {
        self.reveal_with_extent(origin, 2, |dx, dy| !(dx.abs() == 2 && dy.abs() == 2));
    }

    fn reveal_with_extent<F>(&mut self, origin: Location, extent: isize, include: F)
    where
        F: Fn(isize, isize) -> bool,
    {
        let x0 = origin.x as isize;
        let y0 = origin.y as isize;
        for y in (y0 - extent)..=(y0 + extent) {
            if y < 0 || y >= self.height as isize {
                continue;
            }
            for x in (x0 - extent)..=(x0 + extent) {
                if !include(x - x0, y - y0) {
                    continue;
                }
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
        assert!(!exploration.discovered(2, 2));
    }

    #[test]
    fn reveal_marks_every_tile_within_the_radius() {
        let mut exploration = Exploration::new(5, 5);
        exploration.reveal_tiles_at(Location::new(2, 2), 1);
        for y in 1..=3 {
            for x in 1..=3 {
                assert!(exploration.discovered(x, y));
            }
        }
        assert!(!exploration.discovered(0, 0));
        assert!(!exploration.discovered(4, 4));
    }

    #[test]
    fn reveal_wraps_around_east_and_west_but_clamps_north_and_south() {
        let mut exploration = Exploration::new(3, 3);
        exploration.reveal_tiles_at(Location::new(0, 0), 1);
        for y in 0..=1 {
            for x in 0..3 {
                assert!(exploration.discovered(x, y));
            }
        }
        assert!(!exploration.discovered(0, 2));
        assert!(!exploration.discovered(2, 2));
    }

    #[test]
    fn reveal_city_tiles_marks_the_21_tile_footprint() {
        let mut exploration = Exploration::new(5, 5);
        exploration.reveal_tiles_surrounding_city_at(Location::new(2, 2));
        for y in 0..5 {
            for x in 0..5 {
                let corner = (x == 0 || x == 4) && (y == 0 || y == 4);
                assert_eq!(exploration.discovered(x, y), !corner);
            }
        }
    }
}
