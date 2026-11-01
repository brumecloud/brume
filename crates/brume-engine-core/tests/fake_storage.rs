//! An in-memory `Storage` for the engine's tests. Handles share their state, so a test can
//! reopen an engine on the same storage and inspect what was written.

#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use brume_domain::{BlobId, ChangeId, VariableId};
use brume_engine_core::{
    Storage, StorageError, StoredChange, StoredEvent, StoredSession, Transaction,
};
use secrecy::SecretString;

#[derive(Default)]
struct Inner {
    events: Vec<StoredEvent>,
    blobs: BTreeMap<BlobId, Vec<u8>>,
    secrets: BTreeMap<(VariableId, u32), SecretString>,
    staged: BTreeMap<ChangeId, StoredChange>,
    sessions: BTreeMap<[u8; 32], StoredSession>,
    ingress_token: Option<SecretString>,
    commits: usize,
    fail_next_commit: bool,
}

#[derive(Clone, Default)]
pub struct FakeStorage(Rc<RefCell<Inner>>);

impl FakeStorage {
    /// How many transactions were committed.
    pub fn commits(&self) -> usize {
        self.0.borrow().commits
    }

    /// The stored events.
    pub fn events(&self) -> Vec<StoredEvent> {
        self.0.borrow().events.clone()
    }

    /// Changes one byte of a stored event, as an attacker with the disk would.
    pub fn tamper(&self, seq: u64, index: usize) {
        let mut inner = self.0.borrow_mut();
        let event = inner
            .events
            .iter_mut()
            .find(|event| event.seq == seq)
            .expect("the event exists");
        event.bytes[index] ^= 0x01;
    }

    /// Makes the next commit fail.
    pub fn fail_next_commit(&self) {
        self.0.borrow_mut().fail_next_commit = true;
    }

    /// Loses every secret version, as a database restored without them would.
    pub fn forget_secrets(&self) {
        self.0.borrow_mut().secrets.clear();
    }

    /// Loses the tunnel's token, as a database restored without it would.
    pub fn forget_ingress_token(&self) {
        self.0.borrow_mut().ingress_token = None;
    }

    /// Every stored blob.
    pub fn blobs(&self) -> Vec<Vec<u8>> {
        self.0.borrow().blobs.values().cloned().collect()
    }
}

impl Storage for FakeStorage {
    fn load_events(&self) -> Result<Vec<StoredEvent>, StorageError> {
        Ok(self.0.borrow().events.clone())
    }

    fn commit(&mut self, tx: Transaction) -> Result<(), StorageError> {
        let mut inner = self.0.borrow_mut();
        if inner.fail_next_commit {
            inner.fail_next_commit = false;
            return Err(StorageError("disk full".to_owned()));
        }
        let next = inner.events.last().map_or(1, |event| event.seq + 1);
        for (offset, event) in tx.events.iter().enumerate() {
            assert_eq!(event.seq, next + offset as u64, "events are contiguous");
        }
        inner.events.extend(tx.events);
        inner.blobs.extend(tx.blobs);
        for (variable, version, value) in tx.secrets {
            let previous = inner.secrets.insert((variable, version), value);
            assert!(previous.is_none(), "a secret version is written once");
        }
        for change in tx.staged_upserts {
            inner.staged.insert(change.id.clone(), change);
        }
        for change in tx.staged_deletes {
            inner.staged.remove(&change);
        }
        for session in tx.sessions_upserts {
            inner.sessions.insert(session.token_hash, session);
        }
        for session in tx.sessions_deletes {
            inner.sessions.remove(&session);
        }
        if let Some(token) = tx.ingress_token {
            inner.ingress_token = Some(token);
        }
        inner.commits += 1;
        Ok(())
    }

    fn blob(&self, id: &BlobId) -> Result<Option<Vec<u8>>, StorageError> {
        Ok(self.0.borrow().blobs.get(id).cloned())
    }

    fn secret(
        &self,
        variable: &VariableId,
        version: u32,
    ) -> Result<Option<SecretString>, StorageError> {
        Ok(self
            .0
            .borrow()
            .secrets
            .get(&(variable.clone(), version))
            .cloned())
    }

    fn staged_changes(&self) -> Result<Vec<StoredChange>, StorageError> {
        Ok(self.0.borrow().staged.values().cloned().collect())
    }

    fn session(&self, token_hash: &[u8; 32]) -> Result<Option<StoredSession>, StorageError> {
        Ok(self.0.borrow().sessions.get(token_hash).cloned())
    }

    fn ingress_token(&self) -> Result<Option<SecretString>, StorageError> {
        Ok(self.0.borrow().ingress_token.clone())
    }

    fn events_page(
        &self,
        before: Option<u64>,
        limit: u32,
    ) -> Result<Vec<StoredEvent>, StorageError> {
        Ok(self
            .0
            .borrow()
            .events
            .iter()
            .rev()
            .filter(|event| before.is_none_or(|before| event.seq < before))
            .take(limit as usize)
            .cloned()
            .collect())
    }
}
