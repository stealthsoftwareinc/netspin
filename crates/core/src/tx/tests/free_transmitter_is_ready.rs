//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::Executor;
use crate::Moment;
use crate::Task;
use crate::poll;

// A transmitter with free channel space remains immediately ready
// across repeated polls.
#[test]
fn test() {
  let (tx, _rx) = Channel::<i64>::new().pair();
  let mut executor = Executor::new();
  let sender = executor.spawn(async move {
    let first = poll(&[&tx]).await;
    let second = poll(&[&tx]).await;
    (first, second, Task::now())
  });
  executor.run();

  assert_eq!(sender.output(), Some((0, 0, Moment::ZERO)));
}
