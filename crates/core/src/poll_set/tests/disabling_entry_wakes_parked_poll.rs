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

// Disabling an entry wakes a parked poll so its registrations are
// rebuilt without that entry.
#[test]
fn test() {
  let mut executor = Executor::new();
  let poll_set = PollSet::new();
  let (trigger1, signal1) = Signal::pair();
  let (trigger2, signal2) = Signal::pair();
  let signal1 = poll_set.insert(signal1);
  let signal2 = poll_set.insert(signal2);
  let key2 = signal2.key();
  let waiter = executor.spawn(async move {
    let ready = poll_set.poll().await;
    (ready, Task::now())
  });
  let mutator = executor.spawn(async move {
    Task::sleep(Duration::from_nanos(10)).await;
    signal1.disable();
    Task::sleep(Duration::from_nanos(5)).await;
    trigger1.trigger();
    Task::sleep(Duration::from_nanos(5)).await;
    trigger2.trigger();
    signal1
  });
  executor.run();
  assert_eq!(waiter.output().unwrap(), (key2, Moment::from_nanos(20)),);
  assert!(!mutator.output().unwrap().is_enabled());
}
