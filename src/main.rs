mod db;
mod gui;
use crate::gui::run;

fn main() {
    println!("Hello, world!");

    let app_settings = db::AppSettings::new("test.db".into()).unwrap();

    run(app_settings);
}
