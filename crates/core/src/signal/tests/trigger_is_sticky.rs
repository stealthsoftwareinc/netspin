//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use super::super::Signal;
use crate::Executor;
use crate::poll;

// A signal remains ready after being observed.
#[test]
fn test() {
  let (trigger, signal) = Signal::pair();
  trigger.trigger();
  let mut executor = Executor::new();
  let output = executor.spawn(async move {
    (poll(&[&signal]).await, poll(&[&signal]).await)
  });
  executor.run();
  assert_eq!(output.output(), Some((0, 0)));
}
