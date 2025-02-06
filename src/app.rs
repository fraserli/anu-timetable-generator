use std::collections::BTreeMap;

use leptos::prelude::*;

use crate::search::search;
use crate::timetable::{Activity, Class, Course};

type ActivitySelection = Vec<RwSignal<bool>>;

#[component]
pub fn App() -> impl IntoView {
    let session = RwSignal::new(crate::timetable::default_session().to_owned());
    let courses = RwSignal::new(Vec::<(Course, ActivitySelection)>::new());

    view! {
        <h1>"ANU Timetable Generator"</h1>
        <SessionSelector session={session.write_only()} courses={courses.write_only()} />
        <CourseSelector session={session.read_only()} courses={courses} />
        <SearchResults courses={courses.read_only()} />
    }
}

#[component]
fn SessionSelector(
    session: WriteSignal<String>,
    courses: WriteSignal<Vec<(Course, ActivitySelection)>>,
) -> impl IntoView {
    view! {
        <div>
            <label>
                "Session: "
                <select
                    name="session"
                    style="display: inline;"
                    on:input:target={move |ev| {
                        session.set(ev.target().value());
                        courses.write().clear();
                    }}
                >
                    {crate::timetable::sessions()
                        .enumerate()
                        .map(|(i, name)| {
                            view! {
                                <option value={name.to_owned()} selected={i == 0}>
                                    {name.to_owned()}
                                </option>
                            }
                        })
                        .collect_view()}
                </select>
            </label>
        </div>
    }
}

#[component]
fn CourseSelector(
    session: ReadSignal<String>,
    courses: RwSignal<Vec<(Course, ActivitySelection)>>,
) -> impl IntoView {
    let all_courses = LocalResource::new(move || async move {
        let session = &session.read();
        crate::timetable::get_courses(session).await
    });

    let add_course = move |code: String| {
        if let Some(all_courses) = all_courses.get() {
            let course = all_courses.iter().find(|c| c.code == code).unwrap().clone();
            let selected_activities = course
                .activities
                .iter()
                .map(|activity| RwSignal::new(!activity.name.starts_with("Dro")))
                .collect();

            courses.update(|courses| {
                let idx = courses
                    .binary_search_by_key(&course.code, |(c, _)| c.code.clone())
                    .unwrap_or_else(|e| e);
                courses.insert(idx, (course, selected_activities));
            });
        }
    };

    let course_list_view = move || {
        all_courses
            .get()
            .as_deref()
            .unwrap_or(&Vec::new())
            .iter()
            .map(|Course { code, .. }| {
                let disabled = courses.read().iter().any(|(c, _)| c.code == *code);
                view! {
                    <option value={code.to_owned()} disabled={disabled}>
                        {code.to_owned()}
                    </option>
                }
            })
            .collect_view()
    };

    let selected_courses_view = move || {
        courses
            .read()
            .iter()
            .enumerate()
            .map(|(i, (course, selected_activities))| {
                let activities_view = course
                    .activities
                    .iter()
                    .zip(selected_activities.iter())
                    .map(|(activity, &selection)| {
                        view! {
                            <li>
                                <label>
                                    <input type="checkbox" bind:checked={selection} />
                                    {activity.name.to_owned()}
                                    {format!(" ({} option(s))", activity.classes.len())}
                                </label>
                            </li>
                        }
                    })
                    .collect_view();
                view! {
                    <details>
                        <summary>
                            {format!("\t\t{} - {}", course.code, course.name)}
                            <a
                                style="position: relative; float: right;"
                                on:click={move |ev| {
                                    ev.prevent_default();
                                    courses
                                        .update(|courses| {
                                            courses.remove(i);
                                        });
                                }}
                            >
                                "Remove"
                            </a>
                        </summary>
                        <ul>{activities_view}</ul>
                    </details>
                }
            })
            .collect_view()
    };

    // TODO: use something which has better searching instead of <select>
    view! {
        <div>
            <label>
                "Add course: "
                <select
                    name="course"
                    disabled={move || { courses.with(|s| s.len() >= 6) }}
                    style="display: inline;"
                    on:input:target={move |ev| {
                        add_course(ev.target().value());
                        ev.target().set_selected_index(0);
                    }}
                >
                    <option selected disabled hidden>
                        "Select course"
                    </option>
                    {course_list_view}
                </select>
            </label>
            {selected_courses_view}
        </div>
    }
}

#[component]
fn SearchResults(courses: ReadSignal<Vec<(Course, ActivitySelection)>>) -> impl IntoView {
    let result = move || {
        let courses = courses.read();

        let (courses, activities): (Vec<&Course>, Vec<&Activity>) = {
            let mut tmp: Vec<(&Course, &Activity)> =
                courses
                    .iter()
                    .flat_map(|(course, selection)| {
                        course.activities.iter().zip(selection.iter()).filter_map(
                            move |(activity, sel)| sel.get().then_some((course, activity)),
                        )
                    })
                    .collect();

            // Sorting the activities by the number of options reduces backtracking and
            // makes the search run much faster.
            tmp.sort_by_key(|(_, activity)| activity.classes.len());
            tmp.into_iter().unzip()
        };

        if activities.is_empty() {
            "No courses selected".into_any()
        } else {
            let combinations: usize = activities.iter().map(|a| a.classes.len()).product();

            let (classes, searched, score) = search(&activities);

            let mut results: BTreeMap<String, Vec<(&str, &Class)>> = courses
                .iter()
                .map(|course| (course.code.clone(), Vec::new()))
                .collect();

            courses
                .into_iter()
                .zip(activities.iter())
                .zip(classes.iter())
                .for_each(|((course, activity), class)| {
                    results
                        .get_mut(&course.code)
                        .unwrap()
                        .push((&activity.name, class));
                });

            let query: String = results
                .iter()
                .fold(String::new(), |mut acc, (code, classes)| {
                    use std::fmt::Write;
                    let _ = write!(
                        acc,
                        "&{}={}",
                        code,
                        classes
                            .iter()
                            .map(|(activity, class)| format!("{}{}", activity, class.occurrence))
                            .collect::<Vec<String>>()
                            .join(",")
                    );
                    acc
                });

            let url = format!("https://timetable.cssa.club/?y=2025&s=S1{}", query);

            view! {
                <div>{format!("Searched through {searched} / {combinations} combinations")}</div>
                <div>"Score: " {score}</div>
                <div>
                    <a href={url} target="_blank" rel="noreferrer noopener">
                        "Open on CSSA Timetable"
                    </a>
                </div>
            }
            .into_any()
        }
    };

    view! { <div>{result}</div> }
}
