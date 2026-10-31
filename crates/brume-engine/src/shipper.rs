//! Ships the event log to the bucket, in segments: a header line, then the stored bytes of
//! each event, one per line. An object is never overwritten. The checkpoint (`shipper.next_seq`)
//! advances in its own transaction, after a successful put.

use std::path::Path;
use std::time::Duration;

use brume_runtime::object_store::{ObjectStore, PutOutcome};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

use crate::sqlite::connect;

/// The log generation of this milestone.
pub const GENERATION: u32 = 1;

/// The format of a segment.
const SEGMENT_FORMAT: u32 = 1;

/// The most events in one segment.
pub const MAX_SEGMENT_EVENTS: u32 = 1000;

/// How often the shipper looks for new events: at most one segment per period.
pub const SHIP_PERIOD: Duration = Duration::from_secs(1);

/// The first line of a segment.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct SegmentHeader {
    segment: u32,
    generation: u32,
    first_seq: u64,
    count: u64,
}

/// `log/<generation on 10 digits>/<first seq on 20 digits>.seg`.
pub fn segment_key(generation: u32, first_seq: u64) -> String {
    format!("log/{generation:010}/{first_seq:020}.seg")
}

/// A segment: its header line, then each event's bytes and a newline.
pub fn segment_bytes(generation: u32, first_seq: u64, events: &[Vec<u8>]) -> Vec<u8> {
    let header = SegmentHeader {
        segment: SEGMENT_FORMAT,
        generation,
        first_seq,
        count: events.len() as u64,
    };
    let mut bytes = serde_json::to_vec(&header).expect("a header serializes");
    bytes.push(b'\n');
    for event in events {
        bytes.extend_from_slice(event);
        bytes.push(b'\n');
    }
    bytes
}

/// What one round of shipping did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShipOutcome {
    /// Nothing new to ship.
    Idle,
    /// A segment is in the bucket, and the checkpoint moved past it.
    Shipped { first_seq: u64, count: u64 },
    /// Another server's object holds the key: this engine must stop writing.
    Conflict { key: String },
}

/// A failure to read the log, write the checkpoint or reach the bucket. The next round retries.
#[derive(Debug, thiserror::Error)]
pub enum ShipError {
    #[error("the log database failed: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("the bucket failed: {0}")]
    Bucket(#[from] std::io::Error),
    #[error("the log has a gap before event {0}")]
    Gap(u64),
    #[error("a sequence number is out of range")]
    Corrupt,
}

/// Ships segments from its own connection to the log database.
pub struct Shipper<O> {
    connection: Connection,
    store: O,
}

impl<O: ObjectStore> Shipper<O> {
    /// Opens its own connection to the engine's database at `path`.
    pub fn open(path: &Path, store: O) -> Result<Self, ShipError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(Self {
            connection: connect(path)?,
            store,
        })
    }

    fn next_seq(&self) -> Result<u64, ShipError> {
        let next: Option<i64> = self
            .connection
            .query_row("SELECT next_seq FROM shipper WHERE id = 1", [], |row| {
                row.get(0)
            })
            .optional()?;
        next.map_or(Ok(1), u64::try_from)
            .map_err(|_| ShipError::Corrupt)
    }

    fn checkpoint(&mut self, next_seq: u64) -> Result<(), ShipError> {
        let sql = self.connection.transaction()?;
        sql.execute(
            "INSERT INTO shipper (id, next_seq) VALUES (1, ?1)
             ON CONFLICT (id) DO UPDATE SET next_seq = excluded.next_seq",
            params![i64::try_from(next_seq).map_err(|_| ShipError::Corrupt)?],
        )?;
        sql.commit()?;
        Ok(())
    }

    /// Ships the next segment, if there are new events.
    pub fn ship_once(&mut self) -> Result<ShipOutcome, ShipError> {
        let first_seq = self.next_seq()?;
        let events = {
            let mut statement = self
                .connection
                .prepare("SELECT seq, bytes FROM events WHERE seq >= ?1 ORDER BY seq LIMIT ?2")?;
            statement
                .query_map(
                    params![
                        i64::try_from(first_seq).map_err(|_| ShipError::Corrupt)?,
                        MAX_SEGMENT_EVENTS
                    ],
                    |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?)),
                )?
                .collect::<Result<Vec<_>, _>>()?
        };
        if events.is_empty() {
            return Ok(ShipOutcome::Idle);
        }
        for (offset, (seq, _)) in events.iter().enumerate() {
            if u64::try_from(*seq).ok() != Some(first_seq + offset as u64) {
                return Err(ShipError::Gap(first_seq + offset as u64));
            }
        }
        let events = events
            .into_iter()
            .map(|(_, bytes)| bytes)
            .collect::<Vec<_>>();
        let key = segment_key(GENERATION, first_seq);
        let count = match self
            .store
            .put_if_absent(&key, &segment_bytes(GENERATION, first_seq, &events))?
        {
            PutOutcome::Created => events.len() as u64,
            PutOutcome::AlreadyExists => {
                // An earlier run may have put it and crashed before its checkpoint, with fewer
                // events than there are now: it is ours if it is exactly our first ones.
                let existing = self.store.get(&key)?.unwrap_or_default();
                match ours(&existing, first_seq, &events) {
                    Some(count) => count,
                    None => return Ok(ShipOutcome::Conflict { key }),
                }
            }
        };
        self.checkpoint(first_seq + count)?;
        Ok(ShipOutcome::Shipped { first_seq, count })
    }
}

/// The number of events of `existing` if it is the segment of our events from `first_seq`.
fn ours(existing: &[u8], first_seq: u64, events: &[Vec<u8>]) -> Option<u64> {
    let header_end = existing.iter().position(|byte| *byte == b'\n')?;
    let header: SegmentHeader = serde_json::from_slice(&existing[..header_end]).ok()?;
    let count = usize::try_from(header.count).ok()?;
    if header.generation != GENERATION
        || header.first_seq != first_seq
        || count == 0
        || count > events.len()
    {
        return None;
    }
    (segment_bytes(GENERATION, first_seq, &events[..count]) == existing).then_some(header.count)
}
