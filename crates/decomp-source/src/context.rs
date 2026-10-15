//! Operation-local source reuse, inherited with the bounded input scope.
//!
//! Compose source operations with `limited` or `until`. A captured
//! `SourceContext` gives workers the same cache, counters and deadline. Cached
//! identities are hints for avoiding archive expansion, never trusted reads:
//! source authentication rechecks the archive and every pinned file on reuse.
use crate::Source;
use anyhow::{Context, Result, ensure};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tool_execution::bounded;

type Key = (PathBuf, PathBuf, String);
type Pinned = Arc<Mutex<BTreeMap<Key, Source>>>;

thread_local! {
    static CURRENT: RefCell<Option<Pinned>> = const { RefCell::new(None) };
}

struct Restore(Option<Pinned>);
impl Drop for Restore {
    fn drop(&mut self) {
        CURRENT.with(|current| *current.borrow_mut() = self.0.take());
    }
}

/// A captured source operation and its exact bounded input scope.
#[derive(Clone)]
pub struct SourceContext {
    inputs: bounded::SharedScope,
    pinned: Pinned,
}

impl SourceContext {
    /// Enter a worker with the parent's source cache, counters and deadline.
    pub fn enter<T>(&self, action: impl FnOnce() -> Result<T>) -> Result<T> {
        self.inputs.enter(|| {
            let previous = CURRENT.with(|current| -> Result<_> {
                ensure!(current.borrow().is_none(), "source worker already has an active context");
                Ok(current.borrow_mut().replace(self.pinned.clone()))
            })?;
            let _restore = Restore(previous);
            action()
        })
    }
}

pub fn shared_scope() -> Result<SourceContext> {
    let inputs = bounded::shared_scope()?;
    let pinned = CURRENT.with(|current| current.borrow().clone().context("missing shared source context"))?;
    Ok(SourceContext { inputs, pinned })
}

fn with_cache<T>(action: impl FnOnce() -> Result<T>) -> Result<T> {
    if CURRENT.with(|current| current.borrow().is_some()) {
        return action();
    }
    let previous = CURRENT.with(|current| current.borrow_mut().replace(Arc::new(Mutex::new(BTreeMap::new()))));
    let _restore = Restore(previous);
    action()
}

pub fn limited<T>(action: impl FnOnce() -> Result<T>) -> Result<T> {
    bounded::limited(|| with_cache(action))
}

pub fn until<T>(deadline_unix_ms: u64, max_bytes: u64, action: impl FnOnce() -> Result<T>) -> Result<T> {
    bounded::until(deadline_unix_ms, max_bytes, || with_cache(action))
}

pub fn discovery<T>(action: impl FnOnce() -> Result<T>) -> Result<T> {
    bounded::discovery(|| with_cache(action))
}

pub fn discovery_with_limit<T>(max_paths: usize, action: impl FnOnce() -> Result<T>) -> Result<T> {
    bounded::discovery_with_limit(max_paths, || with_cache(action))
}

/// Reconciliation has its own finite budget and therefore a fresh source cache.
pub fn reconcile<T>(max_bytes: u64, action: impl FnOnce() -> Result<T>) -> Result<T> {
    let previous = CURRENT.with(|current| current.borrow_mut().take());
    let _restore = Restore(previous);
    bounded::reconcile(max_bytes, || with_cache(action))
}

pub(crate) fn cached_source(key: &Key) -> Option<Source> {
    CURRENT.with(|current| current.borrow().as_ref().and_then(|pinned| pinned.lock().unwrap().get(key).cloned()))
}

pub(crate) fn cache_source(key: Key, source: Source) {
    CURRENT.with(|current| {
        if let Some(pinned) = &*current.borrow() {
            pinned.lock().unwrap().insert(key, source);
        }
    });
}
