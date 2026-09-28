//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;
use std::rc::Rc;

use crate::Executor;
use crate::PollSet;

// The first task to poll a PollSet becomes its only poller.
#[test]
#[should_panic(
  expected = "A PollSet must not have multiple polling tasks"
)]
fn test() {
  let mut executor = Executor::new();
  let poll_set = Rc::new(PollSet::new());
  executor.spawn({
    let poll_set = poll_set.clone();
    async move {
      let _ = poll_set.poll_with_timeout(Duration::ZERO).await;
    }
  });
  executor.spawn(async move {
    let _ = poll_set.poll_with_timeout(Duration::ZERO).await;
  });
  executor.run();
}
