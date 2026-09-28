//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Executor;
use crate::Moment;
use crate::PollSet;
use crate::Signal;
use crate::Task;

// Enabling a ready entry wakes a poll parked on an empty set.
#[test]
fn test() {
  let mut executor = Executor::new();
  let poll_set = PollSet::new();
  let (trigger, signal) = Signal::pair();
  let signal = poll_set.insert(signal);
  signal.disable();
  let key = signal.key();
  let waiter = executor.spawn(async move {
    let ready = poll_set.poll().await;
    (ready, Task::now())
  });
  let enabler = executor.spawn(async move {
    Task::sleep(Duration::from_nanos(10)).await;
    trigger.trigger();
    signal.enable();
    signal
  });
  executor.run();
  assert_eq!(waiter.output().unwrap(), (key, Moment::from_nanos(10)),);
  assert!(enabler.output().unwrap().is_enabled());
}
