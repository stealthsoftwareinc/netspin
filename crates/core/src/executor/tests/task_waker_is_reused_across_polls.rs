//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::future::Future;
use core::pin::Pin;
use core::task::Context;
use core::task::Poll;
use core::task::Waker;

use crate::Executor;
use crate::Moment;
use crate::executor::SUSPEND;
use crate::executor::Suspend;

struct CheckWaker {
  previous: Option<Waker>,
}

impl Future for CheckWaker {
  type Output = ();

  fn poll(
    mut self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Self::Output> {
    if let Some(previous) = self.previous.take() {
      assert!(previous.will_wake(cx.waker()));
      return Poll::Ready(());
    }
    self.previous = Some(cx.waker().clone());
    SUSPEND.set(Some(Suspend::Pause(Moment::ZERO)));
    Poll::Pending
  }
}

// A task is polled with the same waker after suspending.
#[test]
fn test() {
  let mut executor = Executor::new();
  executor.spawn(CheckWaker { previous: None });
  executor.run();
}
