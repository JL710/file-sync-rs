use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct State {
    current_file: PathBuf,
    total_todo: usize,
    done: usize,
}

impl State {
    pub fn current_file(&self) -> &PathBuf {
        &self.current_file
    }

    pub fn total_todo(&self) -> usize {
        self.total_todo
    }

    pub fn done(&self) -> usize {
        self.done
    }
}

#[derive(Debug, Clone)]
pub struct Syncer {
    target: PathBuf,
    sources: Vec<PathBuf>,
}

impl Syncer {
    pub fn new(sources: Vec<PathBuf>, target: PathBuf) -> Self {
        Self { target, sources }
    }
}

impl Iterator for Syncer {
    type Item = State;

    fn next(&mut self) -> Option<Self::Item> {
        let source = match self.sources.pop() {
            Some(s) => s,
            _ => return None,
        };

        println!("Source: {}", source.to_str().unwrap());
        std::thread::sleep(std::time::Duration::from_secs(1));

        Some(State {
            current_file: source.clone(),
            total_todo: 100,
            done: 1,
        })
    }
}
