mod db;
mod gui;
mod sync;
use crate::gui::run;

fn main() {
    let app_settings = db::AppSettings::new("test.db".into()).unwrap();

    run(app_settings);
}
