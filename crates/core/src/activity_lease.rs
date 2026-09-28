//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Moment;
use crate::Task;
use crate::activity::ActivityStateRef;
use crate::executor::reschedule;

/// A lease that keeps an [`Activity`] active until it is dropped.
///
/// [`Activity`]: crate::Activity
#[must_use]
pub struct ActivityLease {
  state: ActivityStateRef,
}

impl ActivityLease {
  pub(super) fn new(state: ActivityStateRef) -> Self {
    Self { state }
  }
}

impl Drop for ActivityLease {
  fn drop(&mut self) {
    let current_moment = Task::try_now();
    let moment = current_moment.unwrap_or(Moment::ZERO);
    let monitor_task = {
      let mut state = self.state.borrow_mut();
      assert!(
        state.active,
        "netspin_core::ActivityLease::drop(): The activity must be \
         active.",
      );
      state.active = false;
      state.release_generation =
        state.release_generation.checked_add(1).expect(
          "netspin_core::ActivityLease::drop(): The release generation \
           overflowed.",
        );
      state.release_moment = Some(moment);
      state.monitor_task.take()
    };
    if current_moment.is_some()
      && let Some(task_id) = monitor_task
    {
      reschedule(task_id, moment);
    }
  }
}
