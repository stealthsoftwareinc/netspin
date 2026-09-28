//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

pub(crate) mod sealed {
  use crate::moment::Moment;

  pub trait PollTargetOps {
    /// Declares the calling task as the poller task for this object.
    ///
    /// The poller task is the only task that's allowed to poll the
    /// object. The first task to call this method becomes the poller
    /// task for the object, and any other tasks that call this method
    /// will cause a panic when debug assertions are enabled.
    fn declare_poller(&self);

    /// Returns the moment at which the object is ready, if known.
    fn ready_at(&self) -> Option<Moment>;

    /// Registers the calling task to resume when the object becomes
    /// ready, whether the task is parked or sleeping until a deadline.
    fn set_poll_task(&self);

    /// Clears the record set by `set_poll_task()`.
    fn clear_poll_task(&self);
  }
}

impl<P: sealed::PollTargetOps + ?Sized> sealed::PollTargetOps for &P {
  fn declare_poller(&self) {
    P::declare_poller(self)
  }

  fn ready_at(&self) -> Option<crate::Moment> {
    P::ready_at(self)
  }

  fn set_poll_task(&self) {
    P::set_poll_task(self)
  }

  fn clear_poll_task(&self) {
    P::clear_poll_task(self)
  }
}
