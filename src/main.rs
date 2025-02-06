mod app;
mod search;
mod timetable;

use crate::app::App;

fn main() {
    leptos::mount::mount_to_body(App);
}
