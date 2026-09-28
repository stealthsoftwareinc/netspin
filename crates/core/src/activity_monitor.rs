//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::cell::Cell;

use crate::Moment;
use crate::PollTarget;
use crate::Pollable;
use crate::Task;
use crate::TaskId;
use crate::activity::ActivityStateRef;
use crate::poll_target_ops::sealed::PollTargetOps;

/// Observes releases of an [`Activity`].
///
/// The first task to poll this monitor becomes its consumer task.
/// All subsequent polling and release consumption must be performed by
/// that task.
///
/// [`Activity`]: crate::Activity
#[must_use]
pub struct ActivityMonitor {
  observed_generation: Cell<u64>,
  state: ActivityStateRef,
  task_id: Cell<Option<TaskId>>,
}

impl ActivityMonitor {
  pub(super) fn new(state: ActivityStateRef) -> Self {
    Self {
      observed_generation: Cell::new(0),
      state,
      task_id: Cell::new(None),
    }
  }

  /// Consumes a ready release notification.
  ///
  /// Returns whether the activity remains idle.
  /// A newer lease may have begun before its earlier release was
  /// consumed.
  #[must_use]
  pub fn expect_release(&self) -> bool {
    self.declare_poller();
    let state = self.state.borrow();
    assert!(
      state.release_generation != self.observed_generation.get()
        && state
          .release_moment
          .is_some_and(|moment| moment <= Task::now()),
      "netspin_core::ActivityMonitor::expect_release(): A release must \
       be ready.",
    );
    self.observed_generation.set(state.release_generation);
    !state.active
  }

  /// Returns whether an unconsumed release is ready.
  #[must_use]
  pub fn has_release(&self) -> bool {
    let state = self.state.borrow();
    state.release_generation != self.observed_generation.get()
      && state
        .release_moment
        .is_some_and(|moment| moment <= Task::now())
  }

  /// Returns whether a lease currently holds the activity.
  #[must_use]
  pub fn is_active(&self) -> bool {
    self.state.borrow().active
  }
}

impl Pollable for ActivityMonitor {
  fn poll_target(&self) -> PollTarget<'_> {
    PollTarget::new(self)
  }
}

impl PollTargetOps for ActivityMonitor {
  fn declare_poller(&self) {
    if cfg!(debug_assertions) {
      if let Some(task_id) = self.task_id.get() {
        debug_assert!(
          Task::id() == task_id,
          "netspin_core::ActivityMonitor::declare_poller(): An activity \
           monitor must not have multiple consumers.",
        );
      } else {
        self.task_id.set(Some(Task::id()));
      }
    }
  }

  fn ready_at(&self) -> Option<Moment> {
    let state = self.state.borrow();
    (state.release_generation != self.observed_generation.get())
      .then_some(state.release_moment)
      .flatten()
  }

  fn set_poll_task(&self) {
    let mut state = self.state.borrow_mut();
    debug_assert!(
      state
        .monitor_task
        .is_none_or(|task_id| task_id == Task::key()),
      "netspin_core::ActivityMonitor::set_poll_task(): Activity \
       monitors must not have multiple consumers.",
    );
    state.monitor_task = Some(Task::key());
  }

  fn clear_poll_task(&self) {
    self.state.borrow_mut().monitor_task = None;
  }
}

#[cfg(test)]
mod tests;
