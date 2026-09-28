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

// A ready disabled entry is ignored in favor of a later enabled entry.
#[test]
fn test() {
  let mut executor = Executor::new();
  let poll_set = PollSet::new();
  let (trigger1, signal1) = Signal::pair();
  let (trigger2, signal2) = Signal::pair();
  let signal1 = poll_set.insert(signal1);
  let signal2 = poll_set.insert(signal2);
  signal1.disable();
  trigger1.trigger();
  let key2 = signal2.key();
  executor.spawn(async move {
    Task::sleep(Duration::from_nanos(10)).await;
    trigger2.trigger();
  });
  let waiter = executor.spawn(async move {
    let ready = poll_set.poll().await;
    (
      ready,
      signal1.is_triggered(),
      signal2.is_triggered(),
      Task::now(),
    )
  });
  executor.run();
  assert_eq!(
    waiter.output().unwrap(),
    (key2, true, true, Moment::from_nanos(10)),
  );
}
