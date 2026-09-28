//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::cell::RefCell;
use std::rc::Rc;

use crate::ActivityLease;
use crate::ActivityMonitor;
use crate::Moment;
use crate::task_key::TaskKey;

pub(super) struct ActivityState {
  pub(super) active: bool,
  pub(super) release_generation: u64,
  pub(super) release_moment: Option<Moment>,
  pub(super) monitor_task: Option<TaskKey>,
}

pub(super) type ActivityStateRef = Rc<RefCell<ActivityState>>;

/// An exclusively held interval of modeled activity.
///
/// An activity is paired with one [`ActivityMonitor`].
/// Acquiring a lease marks the activity active until that lease is
/// dropped and makes the monitor ready on release.
/// Only one lease may exist at a time.
///
/// [`ActivityMonitor`]: crate::ActivityMonitor
#[must_use]
pub struct Activity {
  state: ActivityStateRef,
}

impl Activity {
  /// Acquires the activity until the returned lease is dropped.
  ///
  /// Panics if the activity is already active.
  pub fn lease(&self) -> ActivityLease {
    let mut state = self.state.borrow_mut();
    assert!(
      !state.active,
      "netspin_core::Activity::lease(): The activity is already active.",
    );
    state.active = true;
    ActivityLease::new(self.state.clone())
  }

  /// Creates an inactive activity and its monitor.
  pub fn pair() -> (Self, ActivityMonitor) {
    let state = Rc::new(RefCell::new(ActivityState {
      active: false,
      release_generation: 0,
      release_moment: None,
      monitor_task: None,
    }));
    (
      Self {
        state: state.clone(),
      },
      ActivityMonitor::new(state),
    )
  }
}
