//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::cell::Cell;
use core::cell::RefCell;
use core::ops::Deref;
use core::time::Duration;
use std::rc::Rc;

use crate::Pollable;
use crate::Rx;
use crate::executor::reschedule;
use crate::moment::Moment;
use crate::task::Task;
use crate::task::TaskId;
use crate::task_key::TaskKey;

/// A key that uniquely identifies an entry within a [`PollSet`].
///
/// [`PollSet`]: crate::PollSet
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PollSetKey(u64);

enum ReadinessScan {
  /// The key of the first ready object.
  Ready(PollSetKey),

  /// The earliest future readiness moment.
  Pending(Moment),

  /// No object has a readiness moment.
  Empty,
}

/// Scans for readiness.
fn scan_readiness(
  entries: &[(PollSetKey, Rc<dyn Pollable>)],
  now: Moment,
) -> ReadinessScan {
  if cfg!(debug_assertions) {
    for (_, object) in entries {
      object.poll_target().declare_poller();
    }
  }
  let mut earliest: Option<Moment> = None;
  for (key, object) in entries {
    let target = object.poll_target();
    if let Some(ready_at) = target.ready_at() {
      if ready_at <= now {
        return ReadinessScan::Ready(*key);
      } else {
        earliest = Some(
          earliest.map_or(ready_at, |earliest| earliest.min(ready_at)),
        );
      }
    }
  }
  earliest.map_or(ReadinessScan::Empty, ReadinessScan::Pending)
}

struct Inner {
  entries: Rc<Vec<(PollSetKey, Rc<dyn Pollable>)>>,
  next_key: u64,
  poll_task: Option<TaskKey>,
}

impl Inner {
  fn interrupt(&mut self) {
    let Some(task_id) = self.poll_task.take() else {
      return;
    };
    if Task::is_running() {
      reschedule(task_id, Task::now());
    }
  }

  fn remove_entry(&mut self, key: PollSetKey) {
    if let Ok(index) = self.entries.binary_search_by_key(&key, |x| x.0)
    {
      Rc::make_mut(&mut self.entries).remove(index);
      self.interrupt();
    }
  }

  fn enter_poll(&mut self) {
    debug_assert!(
      self.poll_task.is_none(),
      "A PollSet must not be polled by more than one task",
    );
    self.poll_task = Some(Task::key());
  }
}

/// A dynamic set of [`Pollable`] objects.
///
/// This is similar to [`netspin_core::poll()`], except the poll set can
/// be modified at any time, even by another task while the polling task
/// is in the middle of a poll operation.
///
/// [`Pollable`]: crate::Pollable
/// [`netspin_core::poll()`]: crate::poll()
pub struct PollSet {
  inner: Rc<RefCell<Inner>>,

  /// The task that's allowed to poll the set.
  ///
  /// This is pinned to the first task that polls the set.
  task_id: Cell<Option<TaskId>>,
}

/// An entry of a [`PollSet`].
///
/// Entries are initially enabled.
/// A disabled entry remains accessible through this handle, but is
/// ignored by the [`PollSet`] until it is enabled again.
/// Dropping a [`PollSetEntry`] permanently removes it from its
/// [`PollSet`].
///
/// [`PollSet`]: crate::PollSet
/// [`PollSetEntry`]: crate::PollSetEntry
#[must_use]
pub struct PollSetEntry<P: Pollable> {
  inner: Rc<RefCell<Inner>>,
  key: PollSetKey,
  object: Rc<P>,
}

impl PollSet {
  #[must_use]
  pub fn new() -> Self {
    Self {
      inner: Rc::new(RefCell::new(Inner {
        entries: Rc::new(Vec::new()),
        next_key: 0,
        poll_task: None,
      })),
      task_id: Cell::new(None),
    }
  }

  fn declare_poller(&self) {
    if let Some(task_id) = self.task_id.get() {
      debug_assert!(
        Task::id() == task_id,
        "A PollSet must not have multiple polling tasks",
      );
    } else {
      self.task_id.set(Some(Task::id()));
    }
  }

  /// Registers `object` with this set.
  pub fn insert<P: Pollable + 'static>(
    &self,
    object: P,
  ) -> PollSetEntry<P> {
    let inner = self.inner.clone();
    let object = Rc::new(object);
    let key = {
      let mut inner = inner.borrow_mut();
      let key = PollSetKey(inner.next_key);
      inner.next_key =
        inner.next_key.checked_add(1).expect("PollSet key overflow");
      Rc::make_mut(&mut inner.entries).push((key, object.clone()));
      inner.interrupt();
      key
    };
    PollSetEntry { inner, key, object }
  }

  async fn poll_helper(
    &self,
    timeout: Option<Duration>,
  ) -> Option<PollSetKey> {
    if cfg!(debug_assertions) {
      self.declare_poller();
    }
    let deadline = timeout.map(|timeout| Task::now() + timeout);
    loop {
      let (ready_at, objects) = {
        let inner = self.inner.borrow();
        let now = Task::now();
        let ready_at = match scan_readiness(&inner.entries, now) {
          ReadinessScan::Ready(key) => return Some(key),
          ReadinessScan::Pending(ready_at) => Some(ready_at),
          ReadinessScan::Empty => None,
        };
        if deadline.is_some_and(|deadline| deadline <= now) {
          return None;
        }
        let objects = inner.entries.clone();
        (ready_at, objects)
      };
      let target = match (ready_at, deadline) {
        (Some(ready_at), Some(deadline)) => {
          Some(ready_at.min(deadline))
        }
        (Some(ready_at), None) => Some(ready_at),
        (None, Some(deadline)) => Some(deadline),
        (None, None) => None,
      };
      for (_, object) in objects.iter() {
        object.poll_target().set_poll_task();
      }
      self.inner.borrow_mut().enter_poll();
      if let Some(target) = target {
        Task::sleep_until(target).await;
      } else {
        Task::park(|_| {}).await;
      }
      for (_, object) in objects.iter() {
        object.poll_target().clear_poll_task();
      }
      self.inner.borrow_mut().poll_task = None;
    }
  }

  /// Waits until one of the stored objects is ready.
  #[must_use]
  pub async fn poll(&self) -> PollSetKey {
    self
      .poll_helper(None)
      .await
      .expect("poll_helper() without a timeout must not return None")
  }

  /// Waits until one of the stored objects is ready or until the given
  /// timeout expires.
  ///
  /// The return value is the key of the ready object.
  /// If multiple objects are ready, the lowest key is returned.
  ///
  /// The return value is `None` if the timeout expires.
  /// If an object is ready at the same moment the timeout expires,
  /// readiness wins.
  ///
  /// Note that returning the lowest key when multiple objects are
  /// ready does not cause starvation, as tasks are always assumed to
  /// progress forward in time, not to produce an unbounded amount of
  /// activity while remaining at a single moment.
  #[must_use]
  pub async fn poll_with_timeout(
    &self,
    timeout: Duration,
  ) -> Option<PollSetKey> {
    self.poll_helper(Some(timeout)).await
  }
}

impl Default for PollSet {
  fn default() -> Self {
    Self::new()
  }
}

impl<P: Pollable> PollSetEntry<P> {
  /// Enables this entry.
  ///
  /// This has no effect if the entry is already enabled.
  /// Enabling an entry interrupts any poll currently in progress so
  /// that its readiness is considered immediately.
  pub fn enable(&self)
  where
    P: 'static,
  {
    let mut inner = self.inner.borrow_mut();
    let index =
      match inner.entries.binary_search_by_key(&self.key, |x| x.0) {
        Ok(_) => return,
        Err(index) => index,
      };
    Rc::make_mut(&mut inner.entries)
      .insert(index, (self.key, self.object.clone()));
    inner.interrupt();
  }

  /// Disables this entry without dropping its object or changing its
  /// key.
  ///
  /// This has no effect if the entry is already disabled.
  /// Disabling an entry interrupts any poll currently in progress so
  /// that the entry is ignored immediately.
  pub fn disable(&self) {
    self.inner.borrow_mut().remove_entry(self.key);
  }

  /// Returns whether this entry is enabled.
  #[must_use]
  pub fn is_enabled(&self) -> bool {
    self
      .inner
      .borrow()
      .entries
      .binary_search_by_key(&self.key, |x| x.0)
      .is_ok()
  }

  /// Returns the key of this entry.
  #[must_use]
  pub fn key(&self) -> PollSetKey {
    self.key
  }
}

impl<T, E: core::error::Error + 'static> PollSetEntry<Rx<T, E>> {
  /// Returns whether this entry's channel is empty.
  #[must_use]
  pub fn is_empty(&self) -> bool {
    self.object.is_empty()
  }
}

impl<P: Pollable> Deref for PollSetEntry<P> {
  type Target = P;

  fn deref(&self) -> &Self::Target {
    &self.object
  }
}

impl<P: Pollable> Drop for PollSetEntry<P> {
  fn drop(&mut self) {
    self.inner.borrow_mut().remove_entry(self.key);
  }
}

#[cfg(test)]
mod tests;
