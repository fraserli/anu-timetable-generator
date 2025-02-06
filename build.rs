use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct RawCourse {
    title: String,
    classes: Vec<RawClass>,
}

#[derive(Deserialize)]
struct RawClass {
    day: usize,
    start: String,
    finish: String,
    weeks: String,
    activity: String,
    occurrence: String,
}

#[derive(Serialize)]
pub struct Course {
    code: String,
    name: String,
    activities: Vec<Activity>,
}

#[derive(Serialize)]
struct Activity {
    name: String,
    classes: Vec<Class>,
}

#[derive(Debug, Serialize)]
struct Class {
    occurrence: u8,
    day: u8,
    start: u16,
    end: u16,
}

impl PartialEq for Class {
    fn eq(&self, other: &Self) -> bool {
        self.day == other.day && self.start == other.start
    }
}

impl Eq for Class {}

impl PartialOrd for Class {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Class {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.day.cmp(&other.day).then(self.start.cmp(&other.start))
    }
}

fn process_data((code, course): (String, RawCourse)) -> Course {
    let name = course
        .title
        .strip_prefix(&format!("{code}\u{00A0}"))
        .unwrap()
        .to_owned();

    let code = code.split('_').next().unwrap().to_owned();

    // TODO: improve filtering of activities based on the weeks
    let activity_names: BTreeSet<String> = course
        .classes
        .iter()
        .filter(|raw_class| raw_class.weeks.contains('\u{2011}') || raw_class.weeks.contains(','))
        .map(|raw_class| raw_class.activity.as_str())
        .map(ToOwned::to_owned)
        .collect();

    let activities = activity_names
        .into_iter()
        .map(|name| {
            let mut classes: Vec<Class> = course
                .classes
                .iter()
                .filter(|c| c.activity == name)
                .map(|class| {
                    let mut start = class.start.split(':').map(|s| s.parse::<u16>().unwrap());
                    let mut end = class.finish.split(':').map(|s| s.parse::<u16>().unwrap());
                    Class {
                        occurrence: class.occurrence.parse().unwrap(),
                        day: class.day.try_into().unwrap(),
                        start: 60 * start.next().unwrap() + start.next().unwrap(),
                        end: 60 * end.next().unwrap() + end.next().unwrap(),
                    }
                })
                .collect();

            classes.sort();
            classes.dedup();

            Activity { name, classes }
        })
        .collect();

    Course {
        code,
        name,
        activities,
    }
}

fn expand_session(session: &str) -> &str {
    match session {
        "S1" => "Semester 1",
        "S2" => "Semester 2",
        _ => session,
    }
}

fn main() {
    let input_dir = PathBuf::from_iter([
        env!("CARGO_MANIFEST_DIR"),
        "anutimetable",
        "public",
        "timetable_data",
    ]);

    let output_dir = PathBuf::from_iter([env!("CARGO_MANIFEST_DIR"), "timetable_data"]);

    let index_file = PathBuf::from_iter([&std::env::var("OUT_DIR").unwrap(), "index.bin"]);

    println!("cargo::rerun-if-changed={}", input_dir.display());
    println!("cargo::rustc-env=INDEX_FILE={}", index_file.display());

    let webroot = option_env!("WEBROOT").unwrap_or_default();

    std::fs::create_dir_all(&output_dir).unwrap();

    let mut index = Vec::new();

    // The input files are arranged as "{input_dir}/{year}/{session}.min.json".

    let mut entries: Vec<_> = std::fs::read_dir(&input_dir).unwrap().collect();
    entries.sort_by_key(|e| e.as_ref().unwrap().path());

    for entry in entries {
        let subdir = entry.unwrap().path();
        let year = subdir.file_name().unwrap().to_str().unwrap();

        let mut entries: Vec<_> = std::fs::read_dir(&subdir).unwrap().collect();
        entries.sort_by_key(|e| e.as_ref().unwrap().path());

        for entry in entries {
            let path = entry.unwrap().path();

            let filename = path.file_name().unwrap().to_str().unwrap();

            if !filename.starts_with("S") || !filename.ends_with(".min.json") {
                continue;
            }

            let session = filename.strip_suffix(".min.json").unwrap().to_owned();

            index.push((
                format!("{} {}", expand_session(&session), year),
                format!("{webroot}/timetable_data/{year}-{session}.bin"),
            ));

            // BTreeMap is used so the courses stay sorted by code.
            let raw_courses: BTreeMap<String, RawCourse> =
                serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();

            let courses: Vec<Course> = raw_courses.into_iter().map(process_data).collect();

            std::fs::write(
                output_dir.join(format!("{year}-{session}.bin")),
                postcard::to_allocvec(&courses).unwrap(),
            )
            .unwrap();
        }
    }

    index.reverse();

    std::fs::write(index_file, postcard::to_allocvec(&index).unwrap()).unwrap();
}
