use std::path::{Path, PathBuf};

pub enum Lang {
    German,
    English,
}

pub fn start_sync(lang: &Lang) -> &'static str {
    match lang {
        Lang::German => "Synchronisierung Starten",
        _ => "Start Synchronisation",
    }
}

pub fn add_file(lang: &Lang) -> &'static str {
    match lang {
        Lang::German => "Datei Hinzufügen",
        _ => "Add File",
    }
}

pub fn add_directory(lang: &Lang) -> &'static str {
    match lang {
        Lang::German => "Ordner Hinzufügen",
        _ => "Add Directory",
    }
}

pub fn set_target(lang: &Lang) -> &'static str {
    match lang {
        Lang::German => "Ziel Setzen",
        _ => "Set Target",
    }
}

pub fn source_exists_error(lang: &Lang, path: PathBuf) -> String {
    match lang {
        Lang::German => format!("Quelle {} existiert bereits.", path.to_str().unwrap()),
        _ => format!("Source {} already exists.", path.to_str().unwrap()),
    }
}

pub fn sources_overlap_error(lang: &Lang, path1: &Path, path2: &Path) -> String {
    match lang {
        Lang::German => format!(
            "Pfade überlappen:\n{}\n{}",
            path1.to_str().unwrap(),
            path2.to_str().unwrap()
        ),
        _ => format!(
            "Paths overlap:\n{}\n{}",
            path1.to_str().unwrap(),
            path2.to_str().unwrap()
        ),
    }
}
