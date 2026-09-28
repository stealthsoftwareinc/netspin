//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Channel;
use crate::Executor;
use crate::Moment;
use crate::PollSet;
use crate::SendOptions;
use crate::Task;

// Disabling the earliest entry reschedules a poll to select from the
// remaining entries.
#[test]
fn test() {
  let mut executor = Executor::new();
  let poll_set = PollSet::new();
  let (tx1, rx1) =
    Channel::<i64>::new().capacity(None).local(false).pair();
  let (tx2, rx2) =
    Channel::<i64>::new().capacity(None).local(false).pair();
  let rx1 = poll_set.insert(rx1);
  let rx2 = poll_set.insert(rx2);
  let key2 = rx2.key();
  executor.spawn(async move {
    let options = SendOptions::new().latency(Duration::from_nanos(50));
    tx1.send_with_options(1, options).await.unwrap();
    let options = SendOptions::new().latency(Duration::from_nanos(100));
    tx2.send_with_options(2, options).await.unwrap();
  });
  let waiter = executor.spawn(async move {
    let ready = poll_set.poll().await;
    let _rx2 = rx2;
    (ready, Task::now())
  });
  let disabler = executor.spawn(async move {
    Task::sleep(Duration::from_nanos(20)).await;
    rx1.disable();
    rx1
  });
  executor.run();
  assert_eq!(waiter.output().unwrap(), (key2, Moment::from_nanos(100)),);
  assert!(!disabler.output().unwrap().is_enabled());
}
