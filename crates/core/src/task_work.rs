//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::future::Future;
use core::pin::Pin;
use core::task::Context;
use core::task::Poll;

use crate::executor::Suspend;
use crate::moment::Moment;
use crate::task::Task;

pub(super) struct TaskWork {
  clock: Moment,
  ready: bool,
}

impl TaskWork {
  pub(super) fn new(clock: Moment) -> Self {
    Self {
      clock,
      ready: false,
    }
  }
}

impl Future for TaskWork {
  type Output = ();

  fn poll(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<()> {
    if self.ready {
      return Poll::Ready(());
    }
    self.ready = true;
    Task::suspend(Suspend::Work(self.clock))
  }
}
