//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Executor;
use crate::Moment;
use crate::PollSet;
use crate::Task;

// Polling an empty set with a timeout returns None at the deadline.
#[test]
fn test() {
  let mut executor = Executor::new();
  let poll_set = PollSet::new();
  let receiver = executor.spawn(async move {
    let ready =
      poll_set.poll_with_timeout(Duration::from_nanos(50)).await;
    (ready, Task::now())
  });
  executor.run();
  assert_eq!(
    receiver.output().unwrap(),
    (None, Moment::from_nanos(50))
  );
}
