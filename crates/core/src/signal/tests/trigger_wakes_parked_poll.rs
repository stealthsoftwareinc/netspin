//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use super::super::Signal;
use crate::Executor;
use crate::Moment;
use crate::Task;
use crate::poll;

// A signal wakes a parked poll at the trigger's moment.
#[test]
fn test() {
  let (trigger, signal) = Signal::pair();
  let mut executor = Executor::new();
  let output = executor.spawn(async move {
    assert_eq!(poll(&[&signal]).await, 0);
    Task::now()
  });
  executor.spawn(async move {
    Task::sleep(Duration::from_nanos(100)).await;
    trigger.trigger();
  });
  executor.run();
  assert_eq!(output.output(), Some(Moment::from_nanos(100)));
}
