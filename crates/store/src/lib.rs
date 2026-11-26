//! What temet-work keeps between runs: the event log. Every event the daemon
//! applies is appended, one JSON line each, with its number and the time the
//! daemon stamped on it. The loop's state is rebuilt by replaying the log.
//!
//! A last line cut short by a crash is left out when reading, so a crash never
//! stops the daemon from starting again.

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use temet_work_core::{Event, Journal, Part, World};

/// One logged event.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    /// Its number, from 1, with no gaps.
    pub seq: u64,
    /// When the daemon applied it, in milliseconds since the Unix epoch.
    pub at_ms: u64,
    /// The event.
    pub event: Event,
}

/// Why the log could not be read or written.
#[derive(Debug)]
pub enum StoreError {
    /// The file system refused.
    Io(std::io::Error),
    /// A line other than the last is not a valid entry; the log is damaged.
    Damaged {
        /// The line's number, from 1.
        line: usize,
        /// What was wrong with it.
        why: String,
    },
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "event log: {e}"),
            Self::Damaged { line, why } => write!(f, "event log line {line} is damaged: {why}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<std::io::Error> for StoreError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// The event log in one file.
pub struct Log {
    path: PathBuf,
    file: File,
    next: u64,
}

impl Log {
    /// Opens the log at `path`, making it if it does not exist, and returns it
    /// with the entries already in it.
    pub fn open(path: &Path) -> Result<(Self, Vec<Entry>), StoreError> {
        let entries = Self::read(path)?;
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        let next = entries.last().map_or(1, |e| e.seq + 1);
        Ok((
            Self {
                path: path.to_path_buf(),
                file,
                next,
            },
            entries,
        ))
    }

    /// The entries in the log at `path`. A missing file is an empty log; an
    /// unfinished last line is left out.
    pub fn read(path: &Path) -> Result<Vec<Entry>, StoreError> {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(e.into()),
        };
        let lines: Vec<String> = BufReader::new(file).lines().collect::<Result<_, _>>()?;
        let mut entries = Vec::with_capacity(lines.len());
        for (i, line) in lines.iter().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<Entry>(line) {
                Ok(entry) => entries.push(entry),
                // A crash can leave the last line half written.
                Err(_) if i + 1 == lines.len() => break,
                Err(e) => {
                    return Err(StoreError::Damaged {
                        line: i + 1,
                        why: e.to_string(),
                    });
                }
            }
        }
        Ok(entries)
    }

    /// Appends an event stamped `at_ms` and returns its entry once it is on disk.
    pub fn append(&mut self, at_ms: u64, event: &Event) -> Result<Entry, StoreError> {
        let entry = Entry {
            seq: self.next,
            at_ms,
            event: event.clone(),
        };
        let line = serde_json::to_string(&entry).map_err(|e| StoreError::Damaged {
            line: 0,
            why: e.to_string(),
        })?;
        writeln!(self.file, "{line}")?;
        self.file.sync_data()?;
        self.next += 1;
        Ok(entry)
    }

    /// The file the log is in.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Journal for Log {
    fn append(&mut self, at_ms: u64, event: &Event) -> Result<(), String> {
        Log::append(self, at_ms, event)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

/// Rebuilds the loop's state from logged entries, in order.
pub fn replay(entries: &[Entry]) -> World {
    World::replay(entries.iter().map(|e| &e.event)).0
}

/// The store part.
#[derive(Default)]
pub struct Store;

impl Part for Store {
    fn id(&self) -> &'static str {
        "store"
    }
}
