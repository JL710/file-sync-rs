use anyhow::Result;
use self_update::cargo_crate_version;

pub fn update() -> Result<self_update::Status> {
    let status = self_update::backends::github::Update::configure()
        .repo_owner("jl710")
        .repo_name("file-sync-rs")
        .bin_name("file-sync-rs")
        .show_download_progress(false)
        .current_version(cargo_crate_version!())
        .build()?
        .update()?;
    Ok(status)
}
