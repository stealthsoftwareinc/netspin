//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Moment;
use crate::executor::reschedule;
use crate::signal::SignalStateRef;
use crate::task::Task;

/// The trigger for a [`Signal`].
///
/// [`Signal`]: crate::Signal
#[must_use]
pub struct SignalTrigger {
  state: SignalStateRef,
}

impl SignalTrigger {
  pub(super) fn new(state: SignalStateRef) -> Self {
    Self { state }
  }

  /// Triggers the signal.
  ///
  /// The signal becomes ready at the calling task's current moment.
  /// If called outside a task, it becomes ready at [`Moment::ZERO`].
  /// Repeated calls have no effect.
  ///
  /// [`Moment::ZERO`]: crate::Moment::ZERO
  pub fn trigger(&self) {
    let current_moment = Task::try_now();
    let moment = current_moment.unwrap_or(Moment::ZERO);
    let rx_task = {
      let mut state = self.state.borrow_mut();
      if state.moment.is_some() {
        return;
      }
      state.moment = Some(moment);
      state.rx_task.take()
    };
    if current_moment.is_some()
      && let Some(task_id) = rx_task
    {
      reschedule(task_id, moment);
    }
  }
}
