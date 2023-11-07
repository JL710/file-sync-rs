use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct State {
    current_file: PathBuf,
    total: usize,
    done: usize,
}

impl State {
    pub fn current_file(&self) -> &PathBuf {
        &self.current_file
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
    fn work(&self) {
        if self.source.is_file() {
            if !self.target.is_file() || !self.files_are_equal() {
                std::fs::copy(&self.source, &self.target).unwrap();
            }
        } else if !self.target.is_dir() {
            std::fs::create_dir(&self.target).unwrap();
        }
    }

    fn files_are_equal(&self) -> bool {
        let source_file = std::fs::read(&self.source).unwrap();
        let target_file = std::fs::read(&self.target).unwrap();

        if source_file.len() != target_file.len() {
            return false;
        }

        for (source_byte, target_byte) in source_file.iter().zip(target_file.iter()) {
            if source_byte != target_byte {
                return false;
            }
        }

        true
    }
}

#[derive(Debug, Clone)]
pub struct Syncer {
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
        })
    }

    fn resolve_dir(&mut self, job: &Job) {
        self.jobs_todo.push(job.clone());
        for i in std::fs::read_dir(&job.source).unwrap() {
            let entry = i.unwrap().path();
            let new_job = Job {
                target: job.target.join(entry.file_name().unwrap()),
                source: entry,
            };
            if new_job.source.is_file() {
                self.jobs_todo.push(new_job);
            } else {
                self.resolve_dir(&new_job);
            }
        }
    }

    pub fn resolve(&mut self) {
        let jobs = self.jobs_todo.clone();
        self.jobs_todo.clear();
        for job in jobs {
            if job.source.is_file() {
                self.jobs_todo.push(job)
            } else {
                self.resolve_dir(&job);
            }
        }
        self.jobs_todo.reverse();
    }
}

impl Iterator for Syncer {
    type Item = State;

    fn next(&mut self) -> Option<Self::Item> {
        let job = match self.jobs_todo.pop() {
            Some(s) => s,
            _ => return None,
        };

        self.jobs_done.push(job.clone());

        job.work();

        Some(State {
            current_file: job.source.clone(),
            total: self.jobs_todo.len() + self.jobs_done.len(),
            done: self.jobs_done.len(),
        })
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
