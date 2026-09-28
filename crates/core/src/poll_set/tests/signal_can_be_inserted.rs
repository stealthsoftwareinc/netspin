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

// A PollSet accepts a Signal and returns its key when it becomes ready.
#[test]
fn test() {
  let mut executor = Executor::new();
  let poll_set = PollSet::new();
  let (trigger, signal) = Signal::pair();
  let signal = poll_set.insert(signal);
  let key = signal.key();
  executor.spawn(async move {
    Task::sleep(Duration::from_nanos(10)).await;
    trigger.trigger();
  });
  let waiter = executor.spawn(async move {
    let ready = poll_set.poll().await;
    (ready, signal.is_triggered(), Task::now())
  });
  executor.run();
  assert_eq!(
    waiter.output().unwrap(),
    (key, true, Moment::from_nanos(10)),
  );
}
