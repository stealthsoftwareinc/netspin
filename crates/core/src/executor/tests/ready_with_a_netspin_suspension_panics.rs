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

// Returning Ready after registering a NetSpin suspension panics.
#[test]
#[should_panic(
  expected = "Async rule violation: A task returned ready after \
              registering a suspension."
)]
fn test() {
  let mut executor = Executor::new();
  executor.spawn(async {
    let mut sleep = pin!(Task::sleep(Duration::from_nanos(1)));
    poll_fn(|cx| {
      assert!(sleep.as_mut().poll(cx).is_pending());
      Poll::Ready(())
    })
    .await;
  });
  executor.run();
}
