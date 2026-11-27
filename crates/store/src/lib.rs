//! What temet-work keeps between runs: the event log. Every event the daemon
//! applies is appended, one JSON line each, with its number and the time the
//! daemon stamped on it. The loop's state is rebuilt by replaying the log.
//!
//! A line counts once its newline is written: `append` returns only after the
//! whole line is on disk, so a line without one was never applied. Opening the
//! log cuts such a line off, so a crash never stops the daemon from starting
//! again and the next entry starts on a line of its own.

use std::fs::{File, OpenOptions};
use std::io::Write;
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
    /// Another process has the log open for writing.
    Busy(PathBuf),
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
            Self::Busy(path) => write!(
                f,
                "another temet-workd is using the event log at {}",
                path.display()
            ),
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
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        // One writer at a time: two daemons on one log would each append events
        // the other never applies.
        file.try_lock().map_err(|e| match e {
            std::fs::TryLockError::WouldBlock => StoreError::Busy(path.to_path_buf()),
            std::fs::TryLockError::Error(e) => StoreError::Io(e),
        })?;
        let (entries, complete) = Self::scan(path)?;
        if file.metadata()?.len() > complete {
            file.set_len(complete)?;
            file.sync_data()?;
        }
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

    /// The entries in the log at `path`. A missing file is an empty log; a
    /// last line without its newline is left out.
    pub fn read(path: &Path) -> Result<Vec<Entry>, StoreError> {
        Self::scan(path).map(|(entries, _)| entries)
    }

    /// The entries, and how many bytes of the file hold complete lines.
    fn scan(path: &Path) -> Result<(Vec<Entry>, u64), StoreError> {
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok((Vec::new(), 0)),
            Err(e) => return Err(e.into()),
        };
        let mut entries = Vec::new();
        let mut complete = 0;
        for (n, line) in bytes.split_inclusive(|b| *b == b'\n').enumerate() {
            let Some(body) = line.strip_suffix(b"\n") else {
                // No newline: the write never finished, so the event was never applied.
                break;
            };
            if !body.iter().all(u8::is_ascii_whitespace) {
                let entry =
                    serde_json::from_slice::<Entry>(body).map_err(|e| StoreError::Damaged {
                        line: n + 1,
                        why: e.to_string(),
                    })?;
                entries.push(entry);
            }
            complete += line.len() as u64;
        }
        Ok((entries, complete))
    }

    /// Appends an event stamped `at_ms` and returns its entry once it is on disk.
    pub fn append(&mut self, at_ms: u64, event: &Event) -> Result<Entry, StoreError> {
        let entry = Entry {
            seq: self.next,
            at_ms,
            event: event.clone(),
        };
        let mut line = serde_json::to_string(&entry).map_err(|e| StoreError::Damaged {
            line: 0,
            why: e.to_string(),
        })?;
        line.push('\n');
        // One write, so the newline never lands apart from its line.
        self.file.write_all(line.as_bytes())?;
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

/// Rebuilds the loop's state from logged entries, in order, each applied at the
/// time it was logged with, so each issue keeps the time it last changed.
pub fn replay(entries: &[Entry]) -> World {
    replay_onto(World::default(), entries)
}

/// Rebuilds the loop's state from logged entries onto `world`, in order, each
/// applied at the time it was logged with.
pub fn replay_onto(world: World, entries: &[Entry]) -> World {
    world
        .replay_at(entries.iter().map(|e| (e.at_ms, &e.event)))
        .0
}

/// The store part.
#[derive(Default)]
pub struct Store;

impl Part for Store {
    fn id(&self) -> &'static str {
        "store"
    }
}
