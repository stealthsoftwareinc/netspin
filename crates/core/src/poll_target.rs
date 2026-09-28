//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Moment;
use crate::poll_target_ops::sealed::PollTargetOps;

/// An opaque reference to a pollable object.
#[derive(Clone, Copy)]
#[must_use]
pub struct PollTarget<'a> {
  target: &'a dyn PollTargetOps,
}

impl<'a> PollTarget<'a> {
  pub(crate) fn new(target: &'a dyn PollTargetOps) -> Self {
    Self { target }
  }

  pub(crate) fn declare_poller(self) {
    if cfg!(debug_assertions) {
      self.target.declare_poller();
    }
  }

  pub(crate) fn ready_at(self) -> Option<Moment> {
    self.target.ready_at()
  }

  pub(crate) fn set_poll_task(self) {
    self.target.set_poll_task();
  }

  pub(crate) fn clear_poll_task(self) {
    self.target.clear_poll_task();
  }
}
