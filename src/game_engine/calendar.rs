/// Classic Civ's calendar pacing, shrunk as the eras roll on, slowed down so
/// the ancient era stretches out instead of racing through the centuries. Each
/// entry is `(start_year, end_year, years_per_turn)` in astronomical years,
/// where the BC era runs negative (4000 BC is -4000). There is no year 0:
/// reaching astronomical year 0 means the game now shows 1 AD, the familiar
/// jump straight from 25 BC to 1 AD.
const CALENDAR_SCHEDULE: [(i32, i32, i32); 7] = [
    (-4000, -1000, 50),
    (-1000, 0, 25),
    (0, 500, 10),
    (500, 1500, 5),
    (1500, 1750, 2),
    (1750, 2000, 1),
    (2000, 2100, 1),
];

/// The astronomical year (BC negative, 0 reads as 1 AD) that `turn` falls in,
/// counting from turn 1 in 4000 BC. Past the end of the schedule the game
/// keeps advancing one year per turn.
pub(crate) fn calendar_year(turn: u32) -> i32 {
    let mut remaining = turn.saturating_sub(1) as i32;
    let mut year = CALENDAR_SCHEDULE[0].0;
    for &(start, end, years_per_turn) in &CALENDAR_SCHEDULE {
        let steps = (end - start).div_euclid(years_per_turn);
        if remaining <= steps {
            return year + remaining * years_per_turn;
        }
        remaining -= steps;
        year = end;
    }
    year + remaining
}
