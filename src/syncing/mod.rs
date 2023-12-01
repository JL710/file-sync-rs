use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod sync;

/// Takes a path to a target directory.
/// Will look if a file with stats of the last sync exist and returns the data as [`LastSync`].
pub fn get_last_sync(path: PathBuf) -> Result<Option<LastSync>> {
    const LAST_SYNC_FILENAME: &str = ".last_file_sync_rs.json";

    if !path.join(LAST_SYNC_FILENAME).is_file() {
        return Ok(None);
    }

    let file_content = std::fs::read_to_string(path.join(LAST_SYNC_FILENAME))
        .context("failed to read last sync stats file")?;

    Ok(Some(
        serde_json::from_str(&file_content).context("Could not convert to LastSync")?,
    ))
}

#[derive(Serialize, Deserialize)]
pub struct LastSync {
    timestamp: usize,
    sources: Vec<String>,
    target: Vec<String>,
}
