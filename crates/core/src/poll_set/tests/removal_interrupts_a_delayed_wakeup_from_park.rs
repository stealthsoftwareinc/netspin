//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;
use std::rc::Rc;

use crate::Channel;
use crate::Executor;
use crate::Moment;
use crate::PollSet;
use crate::SendOptions;
use crate::Task;

// Removing an entry interrupts a delayed wakeup scheduled while the
// set was parked, so a remaining channel can wake the set earlier.
#[test]
fn test() {
  let mut executor = Executor::new();
  let set = PollSet::new();
  let (first_tx, first_rx) = Channel::new().local(false).pair();
  let first_tx = Rc::new(first_tx);
  let removed_tx = first_tx.clone();
  let (second_tx, second_rx) = Channel::new().local(false).pair();
  let first = set.insert(first_rx);
  let second = set.insert(second_rx);
  let second_key = second.key();
  let receiver = executor.spawn(async move {
    let ready = set.poll().await;
    let value = second.expect_receive().unwrap();
    (ready, value, Task::now())
  });
  executor.spawn(async move {
    Task::sleep(Duration::from_nanos(5)).await;
    first_tx
      .send_with_options(
        1,
        SendOptions::new().latency(Duration::from_nanos(95)),
      )
      .await
      .unwrap();
  });
  executor.spawn(async move {
    Task::sleep(Duration::from_nanos(10)).await;
    drop(first);
    assert!(removed_tx.error().is_none());
    Task::sleep(Duration::from_nanos(5)).await;
    assert!(removed_tx.error().is_some());
    second_tx
      .send_with_options(
        2,
        SendOptions::new().latency(Duration::from_nanos(5)),
      )
      .await
      .unwrap();
  });
  executor.run();

  let received = receiver.output();
  assert_eq!(received, Some((second_key, 2, Moment::from_nanos(20))));
}
