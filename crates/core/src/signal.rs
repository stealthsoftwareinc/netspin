//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::cell::Cell;
use core::cell::RefCell;
use std::rc::Rc;

use crate::Moment;
use crate::PollTarget;
use crate::Pollable;
use crate::SignalTrigger;
use crate::poll_target_ops::sealed::PollTargetOps;
use crate::task::Task;
use crate::task::TaskId;
use crate::task_key::TaskKey;

pub(super) struct SignalState {
  pub(super) moment: Option<Moment>,
  pub(super) rx_task: Option<TaskKey>,
}

pub(super) type SignalStateRef = Rc<RefCell<SignalState>>;

/// A sticky, one-shot signal that can be waited on with [`poll()`].
///
/// [`poll()`]: crate::poll()
#[must_use]
pub struct Signal {
  state: SignalStateRef,
  task_id: Cell<Option<TaskId>>,
}

impl Signal {
  /// Creates an unset signal and its trigger.
  pub fn pair() -> (SignalTrigger, Self) {
    let state = Rc::new(RefCell::new(SignalState {
      moment: None,
      rx_task: None,
    }));
    (
      SignalTrigger::new(state.clone()),
      Self {
        state,
        task_id: Cell::new(None),
      },
    )
  }

  /// Returns whether the signal has been triggered.
  #[must_use]
  pub fn is_triggered(&self) -> bool {
    self.state.borrow().moment.is_some()
  }
}

impl Pollable for Signal {
  fn poll_target(&self) -> PollTarget<'_> {
    PollTarget::new(self)
  }
}

impl PollTargetOps for Signal {
  fn declare_poller(&self) {
    if cfg!(debug_assertions) {
      if let Some(task_id) = self.task_id.get() {
        debug_assert!(
          Task::id() == task_id,
          "A signal must not have multiple receivers",
        );
      } else {
        self.task_id.set(Some(Task::id()));
      }
    }
  }

  fn ready_at(&self) -> Option<Moment> {
    self.state.borrow().moment
  }

  fn set_poll_task(&self) {
    let mut state = self.state.borrow_mut();
    debug_assert!(
      state.rx_task.is_none_or(|task_id| task_id == Task::key()),
      "Signals must not have multiple receivers",
    );
    state.rx_task = Some(Task::key());
  }

  fn clear_poll_task(&self) {
    self.state.borrow_mut().rx_task = None;
  }
}

#[cfg(test)]
mod tests;
