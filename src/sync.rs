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
pub struct Syncer {
    target: PathBuf,
    /// sources that are not done
    sources_todo: Vec<PathBuf>,
    /// sources that are done
    sources_done: Vec<PathBuf>
}

impl Syncer {
    pub fn new(sources: Vec<PathBuf>, target: PathBuf) -> Self {
        Self { target, sources_todo: sources, sources_done: Vec::new() }
    }

    pub fn total(&self) -> usize {
        self.sources_done.len() + self.sources_todo.len()
    }
}

impl Iterator for Syncer {
    type Item = State;

    fn next(&mut self) -> Option<Self::Item> {
        let source = match self.sources_todo.pop() {
            Some(s) => s,
            _ => return None,
        };
        
        self.sources_done.push(source.clone());

        println!("Source: {}", source.to_str().unwrap());
        std::thread::sleep(std::time::Duration::from_secs(1));

        Some(State {
            current_file: source.clone(),
            total: self.sources_todo.len(),
            done: self.sources_done.len(),
        })
    }
}
