use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod sync;

const LAST_SYNC_FILENAME: &str = "last_file_sync_rs.json";

/// Takes a path to a target directory.
/// Will look if a file with stats of the last sync exist and returns the data as [`LastSync`].
pub fn get_last_sync(path: PathBuf) -> Result<Option<LastSync>> {
    if !path.join(LAST_SYNC_FILENAME).is_file() {
        return Ok(None);
    }

    let file_content = std::fs::read_to_string(path.join(LAST_SYNC_FILENAME))
        .context("failed to read last sync stats file")?;

    Ok(Some(
        serde_json::from_str(&file_content).context("Could not convert to LastSync")?,
    ))
}

/// Takes a [`LastSync`] and writes it to the file container the last sync.
/// It will overwrite any old sync information.
pub fn write_last_sync(path: PathBuf, last_sync: &LastSync) -> Result<()> {
    if !path.is_dir() {
        anyhow::bail!("Path is invalid.");
    }

    std::fs::write(
        path.join(LAST_SYNC_FILENAME),
        serde_json::to_string(last_sync)
            .context("Converting to json failed.")?
            .as_bytes(),
    )
    .context("Writing to file failed.")?;
    Ok(())
}

/// The type used for representing a specific point in time.
pub type DateTime = chrono::DateTime<chrono::offset::Utc>;

#[derive(Serialize, Deserialize)]
pub struct LastSync {
    timestamp: DateTime,
    sources: Vec<String>,
    target: String,
}

impl LastSync {
    pub fn new(timestamp: DateTime, sources: Vec<PathBuf>, target: PathBuf) -> Self {
        Self {
            timestamp,
            sources: sources
                .iter()
                .map(|source| source.to_str().unwrap().to_owned())
                .collect(),
            target: target.to_str().unwrap().to_owned(),
        }
    }
}
