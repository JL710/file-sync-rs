use futures::stream::StreamExt;
use std::io::{Read, Seek, Write};
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
    fn work(&self) {
        if self.source.is_file() {
            if self.target.is_file() {
                self.file_work()
            } else {
                std::fs::copy(&self.source, &self.target).unwrap();
            }
        } else if !self.target.is_dir() {
            std::fs::create_dir(&self.target).unwrap();
        }
    }

    fn file_work(&self) {
        const BUFF_SIZE: usize = 4096;

        let mut source_file = std::fs::File::open(&self.source).unwrap();
        let mut target_file = std::fs::File::open(&self.target).unwrap();

        let source_file_metadata = source_file.metadata().unwrap();
        let target_file_metadata = target_file.metadata().unwrap();

        // change permissions if differ
        if source_file_metadata.permissions() != target_file_metadata.permissions() {
            target_file
                .set_permissions(source_file_metadata.permissions())
                .unwrap();
        }

        // change length of the file if differ
        if source_file_metadata.len() != target_file_metadata.len() {
            target_file.set_len(source_file_metadata.len()).unwrap();
        }

        let mut source_buff: [u8; BUFF_SIZE] = [0; BUFF_SIZE]; // using 1024 because thats one page size
        let mut target_buff: [u8; BUFF_SIZE] = [0; BUFF_SIZE];

        loop {
            let could_read_source = source_file.read_exact(&mut source_buff).is_ok();
            let could_read_target = target_file.read_exact(&mut target_buff).is_ok();

            if could_read_source && could_read_target {
                if source_buff != target_buff {
                    // go back with cursor and overwrite content
                    target_file
                        .seek(std::io::SeekFrom::Current(-(BUFF_SIZE as i64)))
                        .unwrap();
                    target_file.write_all(&source_buff).unwrap();
                }
            } else {
                // check the tail and overwrite if needed
                let mut source_tail: Vec<u8> = Vec::with_capacity(BUFF_SIZE - 1);
                let mut target_tail: Vec<u8> = Vec::with_capacity(BUFF_SIZE - 1);
                source_file.read_to_end(&mut source_tail).unwrap();
                target_file.read_to_end(&mut target_tail).unwrap();
                if source_tail != target_tail {
                    target_file
                        .seek(std::io::SeekFrom::Current(-(source_tail.len() as i64)))
                        .unwrap();
                    target_file.write_all(&source_tail).unwrap();
                }
                break;
            }
        }
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

    fn resolve(&mut self) {
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

    pub async fn prepare(&mut self) {
        // resolve dirs
        tokio::task::block_in_place(move || self.resolve());

        // TODO: write status into file
    }

    pub async fn async_next(&mut self) -> Option<State> {
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
                job.work();
                job
            });
            futures.push(future);
        }

        // wait for them to finish executing
        while let Some(Ok(job)) = futures.next().await {
            self.jobs_done.push(job);
        }

        let done_len = self.jobs_done.len();
        Some(State {
            current_work: current_files,
            total: self.jobs_todo.len() + done_len,
            done: done_len,
        })
    }
}

impl Iterator for Syncer {
    type Item = State;

    fn next(&mut self) -> Option<Self::Item> {
        let job = match self.jobs_todo.pop() {
            Some(s) => s,
            _ => return None,
        };

        job.work();

        let current_file = job.source.clone();

        self.jobs_done.push(job);

        Some(State {
            current_work: vec![current_file],
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
