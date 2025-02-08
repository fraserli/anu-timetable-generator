use std::collections::BinaryHeap;

use crate::timetable::{Activity, Class, Course, Timetable};

struct SearchResult<'a> {
    number: usize,
    classes: Vec<&'a Class>,
    score: i64,
}

pub fn search<'a>(courses: &[(&'a Course, Vec<bool>)]) -> (Vec<Timetable<'a>>, usize, usize) {
    let mut activities: Vec<(&Activity, &str)> = courses
        .iter()
        .flat_map(|(course, selected)| {
            course
                .activities
                .iter()
                .zip(selected.iter())
                .filter(|(_, is_selected)| **is_selected)
                .map(|(activity, _)| (activity, course.code.as_ref()))
        })
        .collect();

    if activities.is_empty() {
        return (Vec::new(), 0, 0);
    }

    // Sorting the activities by the number of options reduces backtracking and
    // makes the search run much faster.
    activities.sort_by_key(|(activity, _)| activity.classes.len());

    let (activities, course_codes): (Vec<&Activity>, Vec<&str>) = activities.into_iter().unzip();

    // Find optimal classes using recursive backtracking.
    fn f<'a>(
        activities: &[&'a Activity],
        current_classes: &mut Vec<&'a Class>,
        results: &mut BinaryHeap<SearchResult<'a>>,
        searched: &mut usize,
    ) {
        let score = eval(current_classes);

        if results.peek().is_some_and(|r| r.score > score + 10) {
            return;
        }

        if current_classes.len() < activities.len() {
            for class in activities[current_classes.len()].classes.iter() {
                current_classes.push(class);
                f(activities, current_classes, results, searched);
                current_classes.pop();
            }
        } else {
            results.push(SearchResult {
                number: *searched,
                classes: current_classes.to_owned(),
                score,
            });
            *searched += 1;
            if results.len() > 25 {
                results.pop();
            }
        }
    }

    let mut searched = 0;
    let mut results = BinaryHeap::new();

    f(&activities, &mut Vec::new(), &mut results, &mut searched);

    let timetables = results.into_sorted_vec().into_iter().map(|result| {
        let mut classes: Vec<(&Class, (&&str, &&Activity))> = result
            .classes
            .into_iter()
            .zip(course_codes.iter().zip(activities.iter()))
            .collect();
        classes.sort_by(|a, b| a.1 .1.name.cmp(&b.1 .1.name));

        Timetable {
            score: result.score,
            number: result.number,
            courses: courses
                .iter()
                .map(|(course, _)| {
                    let classes = classes
                        .iter()
                        .filter_map(|(class, (code, activity))| {
                            (course.code == **code).then_some((activity.name.clone(), *class))
                        })
                        .collect();

                    (course.code.to_owned(), classes)
                })
                .collect(),
        }
    });

    let total_combinations: usize = activities.iter().map(|a| a.classes.len()).product();

    (timetables.collect(), searched, total_combinations)
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

impl Ord for SearchResult<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.score.cmp(&self.score)
    }
}

impl PartialOrd for SearchResult<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for SearchResult<'_> {}

impl PartialEq for SearchResult<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}
