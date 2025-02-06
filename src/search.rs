use crate::timetable::{Activity, Class};

// TODO: add support for returning multiple results
pub fn search<'a>(activities: &'a [&'a Activity]) -> (Vec<&'a Class>, usize, i64) {
    // Find optimal classes using recursive backtracking.
    fn f<'a>(
        activities: &[&'a Activity],
        current_classes: &mut Vec<&'a Class>,
        searched: &mut usize,
        result: &mut Option<(i64, Vec<&'a Class>)>,
    ) {
        let score = eval(current_classes);

        // TODO: allow user configuration of pruning aggressiveness
        if result.as_ref().is_some_and(|&(s, _)| s > score + 25) {
            return;
        }

        if current_classes.len() < activities.len() {
            for class in activities[current_classes.len()].classes.iter() {
                current_classes.push(class);
                f(activities, current_classes, searched, result);
                current_classes.pop();
            }
        } else {
            if result.as_ref().is_none_or(|&(s, _)| score > s) {
                let _ = result.insert((score, current_classes.to_owned()));
            }
            *searched += 1;
        }
    }

    let mut searched = 0;
    let mut result = None;

    f(activities, &mut Vec::new(), &mut searched, &mut result);

    let (score, classes) = result.unwrap_or_default();

    (classes, searched, score)
}

// TODO: allow user customisation of the constants
fn eval(classes: &[&Class]) -> i64 {
    // The occupancy of each day is stored as a bitset. Each 10 minute interval from
    // 4:00 to 24:00 is represented by one bit (120 total), starting from the least
    // significant bit.
    let mut timetable = [0u128; 5];
    let mut collisions = 0;

    for class in classes {
        let num_bits = (class.end - class.start) as u32 / 10;
        let start_offset = (class.start - 4 * 60) / 10;

        let mask = (u128::MAX >> (u128::BITS - num_bits)) << start_offset;

        collisions += (mask & timetable[class.day as usize]).count_ones() as i64;
        timetable[class.day as usize] |= mask;
    }

    let mut score = -collisions * 10;

    for day in timetable {
        if day == 0 {
            score += 20;
        } else {
            let before_preferred = !(u128::MAX << ((10 - 4) * 6)); // 10:00
            let after_preferred = u128::MAX << ((17 - 4) * 6); // 17:00
            score -= (day & (before_preferred | after_preferred)).count_ones() as i64;
        }
    }

    score
}
