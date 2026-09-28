//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::future::Future;
use core::future::poll_fn;
use core::pin::pin;
use core::task::Poll;
use core::time::Duration;

use crate::Executor;
use crate::Task;

// Registering multiple NetSpin suspensions during one task poll panics.
#[test]
#[should_panic(
  expected = "Async rule violation: A task registered more than one \
              suspension during a single poll."
)]
fn test() {
  let mut executor = Executor::new();
  executor.spawn(async {
    let mut first = pin!(Task::sleep(Duration::from_nanos(1)));
    let mut second = pin!(Task::sleep(Duration::from_nanos(2)));
    poll_fn(|cx| {
      assert!(first.as_mut().poll(cx).is_pending());
      assert!(second.as_mut().poll(cx).is_pending());
      Poll::<()>::Pending
    })
    .await;
  });
  executor.run();
}
