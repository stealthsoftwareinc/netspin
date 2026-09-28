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

// Enabling an earlier entry reschedules a poll sleeping for a later
// readiness moment.
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
  rx1.disable();
  let key1 = rx1.key();
  executor.spawn(async move {
    let options = SendOptions::new().latency(Duration::from_nanos(30));
    tx1.send_with_options(1, options).await.unwrap();
    let options = SendOptions::new().latency(Duration::from_nanos(100));
    tx2.send_with_options(2, options).await.unwrap();
  });
  let waiter = executor.spawn(async move {
    let ready = poll_set.poll().await;
    let _rx2 = rx2;
    (ready, Task::now())
  });
  let enabler = executor.spawn(async move {
    Task::sleep(Duration::from_nanos(20)).await;
    rx1.enable();
    rx1
  });
  executor.run();
  assert_eq!(waiter.output().unwrap(), (key1, Moment::from_nanos(30)),);
  assert!(enabler.output().unwrap().is_enabled());
}
