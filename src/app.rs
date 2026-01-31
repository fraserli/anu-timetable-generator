use leptos::prelude::*;

use crate::search::search;
use crate::timetable::Course;

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

        let courses: Vec<(&Course, Vec<bool>)> = courses
            .iter()
            .map(|(course, selected)| (course, selected.iter().map(|s| s.get()).collect()))
            .collect();

        let (timetables, searched, total) = search(&courses);

        if timetables.is_empty() {
            ().into_any()
        } else {
            let timetables_view = timetables
                .into_iter()
                .map(|timetable| {
                    let url = timetable.url("2026", "S1");
                    view! {
                        <blockquote>
                            <b>"Timetable #" {timetable.number}</b>
                            " ("
                            <a target="_blank" rel="noreferrer noopener" href={url}>
                                "Open in CSSA Timetable 🚀"
                            </a>
                            ")"
                            <br />
                            "Score: "
                            {timetable.score}
                            <br />
                        </blockquote>
                    }
                })
                .collect_view();

            view! {
                <div>
                    <p>"Considered " {searched} "/" {total} " combinations."</p>
                    {timetables_view}
                </div>
            }
            .into_any()
        }
    };

    view! { <div>{result}</div> }
}
