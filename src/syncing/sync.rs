use anyhow::{Context, Result};
use futures::stream::StreamExt;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct State {
    current_work: Vec<PathBuf>,
    total: usize,
    done: usize,
}

impl State {
    pub fn current_work(&self) -> &Vec<PathBuf> {
        &self.current_work
    }

    pub fn total(&self) -> usize {
        self.total
    }

    pub fn done(&self) -> usize {
        self.done
    }
}

#[derive(Debug, Clone)]
struct Job {
    source: PathBuf,
    target: PathBuf,
}

impl Job {
    fn work(&self) -> Result<()> {
        if self.source.is_file() {
            if self.target.is_file() {
                self.file_work().context("failed to do file work")?;
            } else {
                std::fs::copy(&self.source, &self.target).with_context(|| {
                    format!("Could not copy file {:?} to {:?}", self.source, self.target)
                })?;
                std::fs::set_permissions(
                    &self.target,
                    std::fs::metadata(&self.source)?.permissions(),
                )
                .context(format!("Could not set permissions for {:?}", self.target))?;
            }
        } else if !self.target.is_dir() {
            std::fs::create_dir(&self.target)
                .context(format!("Could not create directory {:?}", self.target))?;
        }
        Ok(())
    }

    fn file_work(&self) -> Result<()> {
        if std::fs::metadata(&self.target)?.permissions().readonly() {
            let mut perms = std::fs::metadata(&self.target)?.permissions();
            #[allow(clippy::permissions_set_readonly_false)]
            perms.set_readonly(false);
            std::fs::set_permissions(&self.target, perms)
                .context(format!("Could not set permissions for {:?}", self.target))?;
        }

        let mut source_file = std::fs::OpenOptions::new()
            .read(true)
            .open(&self.source)
            .context(format!("Could not open source file {:?}", self.source))?;
        let mut target_file = std::fs::OpenOptions::new()
            .write(true)
            .read(true)
            .open(&self.target)
            .context(format!("Could not open target file {:?}", self.target))?;

        let source_file_metadata = source_file.metadata().context(format!(
            "Could query metadata of source file {:?}",
            self.source
        ))?;
        let target_file_metadata = target_file.metadata().context(format!(
            "Could query metadata of target file {:?}",
            self.source
        ))?;

        // change permissions if differ
        if source_file_metadata.permissions() != target_file_metadata.permissions() {
            let mut perms = source_file_metadata.permissions();
            #[allow(clippy::permissions_set_readonly_false)]
            perms.set_readonly(false);
            target_file.set_permissions(perms).context(format!(
                "Could not set target file metadata for {:?}",
                self.target
            ))?;
        }

        // change length of the file if differ
        if source_file_metadata.len() != target_file_metadata.len() {
            target_file
                .set_len(source_file_metadata.len())
                .context(format!(
                    "Could not set target file length for {:?}",
                    self.target
                ))?;
        }

        // read content of files
        let mut source_file_content = Vec::with_capacity(source_file_metadata.len() as usize);
        let mut target_file_content = Vec::with_capacity(target_file_metadata.len() as usize);
        source_file
            .read_to_end(&mut source_file_content)
            .context(format!("Could not read file {:?}", self.source))?;
        target_file
            .read_to_end(&mut target_file_content)
            .context(format!("Could not read file {:?}", self.target))?;

        // return if files are equal
        if source_file_content == target_file_content {
            return Ok(());
        }
        // write all file content
        target_file
            .write_all(&source_file_content)
            .context(format!("Could not write to file {:?}", self.target))?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Syncer {
    target_root: PathBuf,
    /// jobs that are not done
    jobs_todo: Vec<Job>,
    /// jobs  that are done
    jobs_done: Vec<Job>,
}

impl Syncer {
    pub fn new(sources: Vec<PathBuf>, target: PathBuf) -> Result<Self, InvalidSyncerParameters> {
        valid_syncer_parameters(&sources, &target)?;
        Ok(Self {
            jobs_todo: sources
                .iter()
                .map(|source| Job {
                    source: source.clone(),
                    target: target.join(source.file_name().unwrap()),
                })
                .collect(),
            jobs_done: Vec::new(),
            target_root: target,
        })
    }

    fn resolve_dir(&mut self, job: &Job) -> Result<()> {
        self.jobs_todo.push(job.clone());
        for i in std::fs::read_dir(&job.source)? {
            let i = i?;
            if !i.file_type()?.is_file() && !i.file_type()?.is_dir() {
                continue;
            }
            let entry = i.path();
            let new_job = Job {
                target: job.target.join(entry.file_name().unwrap()),
                source: entry,
            };
            if new_job.source.is_file() {
                self.jobs_todo.push(new_job);
            } else {
                self.resolve_dir(&new_job)
                    .with_context(|| format!("failed to resolve dir for job {:?}", new_job))?;
            }
        }
        Ok(())
    }

    fn resolve(&mut self) -> Result<()> {
        let jobs = self.jobs_todo.clone();
        self.jobs_todo.clear();
        for job in jobs {
            if job.source.is_file() {
                self.jobs_todo.push(job)
            } else {
                self.resolve_dir(&job)
                    .with_context(|| format!("failed to resolve dir for job {:?}", job))?;
            }
        }
        self.jobs_todo.reverse();
        Ok(())
    }

    pub async fn prepare(&mut self) -> Result<()> {
        // write status into file
        super::write_last_sync(
            self.target_root.clone(),
            &super::LastSync::new(
                chrono::offset::Utc::now(),
                self.jobs_todo
                    .iter()
                    .map(|job| job.source.clone())
                    .collect(),
                self.target_root.clone(),
            ),
        )
        .context("Updating the last sync file failed")?;

        // resolve dirs
        tokio::task::block_in_place(move || self.resolve())?;

        Ok(())
    }

    pub async fn async_next(&mut self) -> Option<Result<State>> {
        let mut current_files: Vec<PathBuf> = Vec::new();
        let mut futures = futures::stream::FuturesUnordered::new();

        // gather next x jobs that can be executed concurrently
        for _ in 0..10 {
            let job = match self.jobs_todo.pop() {
                Some(j) => j,
                _ => return None,
            };

            if !current_files.is_empty() && job.source.starts_with(current_files.last().unwrap()) {
                self.jobs_todo.push(job);
                break;
            }

            current_files.push(job.source.clone());

            let future = tokio::task::spawn_blocking(move || {
                let res = job.work();
                (res, job)
            });
            futures.push(future);
        }

        // wait for them to finish executing
        while let Some(Ok(future)) = futures.next().await {
            if let Err(err) = future.0 {
                return Some(Err(err));
            }
            self.jobs_done.push(future.1);
        }

        let done_len = self.jobs_done.len();
        Some(Ok(State {
            current_work: current_files,
            total: self.jobs_todo.len() + done_len,
            done: done_len,
        }))
    }
}

impl Iterator for Syncer {
    type Item = Result<State>;

    fn next(&mut self) -> Option<Self::Item> {
        let job = match self.jobs_todo.pop() {
            Some(s) => s,
            _ => return None,
        };

        let job_res = job.work();
        if let Err(err) = job_res {
            return Some(Err(err));
        }

        let current_file = job.source.clone();

        self.jobs_done.push(job);

        Some(Ok(State {
            current_work: vec![current_file],
            total: self.jobs_todo.len() + self.jobs_done.len(),
            done: self.jobs_done.len(),
        }))
    }
}

fn valid_syncer_parameters(
    sources: &Vec<PathBuf>,
    target: &PathBuf,
) -> Result<(), InvalidSyncerParameters> {
    for source in sources {
        if !source.is_dir() && !source.is_file() {
            // check if source exists
            return Err(InvalidSyncerParameters::SourceDoesNotExist(source.clone()));
        } else if source.starts_with(target) {
            // check if source is in target
            return Err(InvalidSyncerParameters::SourceInTarget(source.clone()));
        } else if target.starts_with(source) {
            // check if target is in source
            return Err(InvalidSyncerParameters::TargetInSource(source.clone()));
        }
    }
    Ok(())
}

#[derive(Debug)]
pub enum InvalidSyncerParameters {
    SourceDoesNotExist(PathBuf),
    TargetInSource(PathBuf),
    SourceInTarget(PathBuf),
}
