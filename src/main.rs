mod db;
mod gui;
mod syncing;
use crate::gui::run;

#[cfg(debug_assertions)]
fn get_db_path() -> String {
    String::from("development.db")
}

#[cfg(not(debug_assertions))]
fn get_db_path() -> String {
    let app_data_dir = dirs::data_dir().unwrap().join("file-sync-rs");
    if !app_data_dir.is_dir() {
        std::fs::create_dir(&app_data_dir).unwrap();
    }
    app_data_dir.join("data.db").to_str().unwrap().into()
}

fn main() {
    let app_settings = db::AppSettings::new(get_db_path().into()).unwrap();

    run(app_settings);
}
