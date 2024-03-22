use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Lang {
    German,
    English,
}

impl From<&str> for Lang {
    fn from(value: &str) -> Self {
        match value {
            "German" => Self::German,
            _ => Self::English,
        }
    }
}

impl From<&Lang> for String {
    fn from(value: &Lang) -> Self {
        match value {
            Lang::German => "German",
            _ => "English",
        }
        .to_owned()
    }
}

pub fn start_sync(lang: &Lang) -> &'static str {
    match lang {
        Lang::German => "Synchronisierung Starten",
        _ => "Start Synchronisation",
    }
}

pub fn source_block_label(lang: &Lang) -> &'static str {
    match lang {
        Lang::German => "Quellen",
        _ => "Sources",
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

pub fn target_does_not_exist_error(lang: &Lang) -> String {
    match lang {
        Lang::German => "Es ist kein Zielverzeichnis eingestellt.",
        _ => "No target directory is given.",
    }
    .to_owned()
}

pub fn sources_does_not_exist_error(lang: &Lang) -> String {
    match lang {
        Lang::German => "Es sind keine Verzeichnisse zum synchronisieren eingestellt.",
        _ => "No source directories given.",
    }
    .to_owned()
}

pub fn source_does_not_exist_error(lang: &Lang, source: &Path) -> String {
    match lang {
        Lang::German => format!("Die Quelle {} existiert nicht.", source.to_str().unwrap()),
        _ => format!("Source {} does not exist.", source.to_str().unwrap()),
    }
}

pub fn source_in_target_error(lang: &Lang, source: &Path) -> String {
    match lang {
        Lang::German => format!(
            "Die Quelle {} befindet sich im Zielverzeichnis",
            source.to_str().unwrap()
        ),
        _ => format!(
            "The source {} is located inside the target directory",
            source.to_str().unwrap()
        ),
    }
}

pub fn target_in_source_error(lang: &Lang, source: &Path) -> String {
    match lang {
        Lang::German => format!(
            "Das Zielverzeichnis befindet sich in diesem Quellverzeichnis: {} .",
            source.to_str().unwrap()
        ),
        _ => format!(
            "The target directory located inside in this source directory: {} .",
            source.to_str().unwrap()
        ),
    }
}

pub fn last_sync(lang: &Lang) -> String {
    match lang {
        Lang::German => "Letzte Synchronisierung",
        _ => "Last Synchronisation",
    }
    .into()
}

pub fn app_update_finished_description(
    lang: &Lang,
    version: &str,
    already_up_to_date: bool,
) -> String {
    match lang {
        Lang::German => {
            if already_up_to_date {
                String::from("Das Programm ist bereits auf dem aktuellsten Stand.")
            } else {
                format!(
                    "Update auf Version {} beendet. Das Programm kann neugestartet werden.",
                    version
                )
            }
        }
        _ => {
            if already_up_to_date {
                String::from("The application is already up to date.")
            } else {
                format!(
                    "Update to version {} finished. Application can be restarted.",
                    version
                )
            }
        }
    }
}
